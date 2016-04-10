/*
  Copyright 2016 Robert Lathrop

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.*/

extern crate mio;
extern crate time;
extern crate slab;

use game::gamemap::MapScreen;
use game::Game;
use game::characters::player::Player;
use game::gamemap::Commandable;
use conn::api::Api;


use mio::tcp::*;
use mio::{TryRead, TryWrite};
use mio::util::Slab;
use mio::buf::ByteBuf;
use mio::buf::Buf;
use self::slab::Index;

use flate2::Compression;
use flate2::write::ZlibEncoder;

use std::io::prelude::*;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, channel};
use std;

/// This module contains all of the client facing code. It handles all of the MIO stuff, and user
/// states and such.
///
/// There are two main structs here. Server & Connection. Server is of traight Handler, and
/// implements the mio callbacks. Connection implements the client RPC API.
///
/// It also declares an enum to handle the connection state, and the Msg protocol used for comms
/// between the connections and game loop.

//Setting the server as the first token
pub const SERVER: mio::Token = mio::Token(0);
pub const TIMEOUT: mio::Token = mio::Token(1);

/// enum for the current state of the connection. Not Logged in, Logged in, and Closed.
enum State {
    Closed,
    NotLoggedIn,
    LoggedIn,
}

/// enum for passing messages between connection & game loop. These are handled in the notify
/// method of the mio Handler
pub enum Msg {
    Command(mio::Token, String),
    SendCommand(mio::Token, Sender<Msg>),
    TextOutput(mio::Token, u8, String),
    Shout(String),
    Screen(mio::Token, MapScreen),
    Hp(mio::Token, i32),
}

/// This server maintain holds the tcp server, as well as a collection of all the current client
/// collections. Additionally, it holds the main game struct.
pub struct Server {
    server: TcpListener,
    //Tried removing the Arc here
    connections: Slab<Connection>,
    games: Arc<RefCell<Game>>,
    recv: Receiver<Msg>,
    send: Sender<Msg>,
}

impl Server {
    /// Declares a new server with a tcp connection
    pub fn new(tcp: TcpListener) -> Server {
        let slab = Slab::new_starting_at(mio::Token(2), 1024);
        let (s, r) = channel::<Msg>();
        Server {
            server: tcp,
            connections: slab,
            games: Arc::new(RefCell::new(Game::new(s.clone()))),
            send: s,
            recv: r,
        }
    }
}

impl mio::Handler for Server {
    type Timeout = mio::Token;
    type Message = ();

    fn timeout(&mut self, event_loop: &mut mio::EventLoop<Server>, timeout: mio::Token) {
        loop {
            match self.recv.try_recv() {
                Ok(msg) => {
                    match msg {
                        Msg::TextOutput(token, result, message) => {
                            // Write message
                            if self.connections.contains(token) {
                                self.connections[token].write_text_out(result, &message);
                            }
                            if self.connections.contains(token) {
                                self.connections[token].reregister_writable(event_loop);
                            }
                        },
                        Msg::Screen(token, screen) => {
                            //Write screen
                            if self.connections.contains(token) {
                                self.connections[token].write_zipped_screen(screen);
                            }
                            if self.connections.contains(token) {
                                self.connections[token].reregister_writable(event_loop);
                            }
                        },
                        Msg::Hp(token, hp) => {
                            if self.connections.contains(token) {
                                self.connections[token].write_stat_all(hp, 500, 100, 100, 25, 1000000, 3000000, 6, 10);
                            }
                            if self.connections.contains(token) {
                                self.connections[token].reregister_writable(event_loop);
                            }
                        },
                        Msg::Shout(msg) => {
                            let mut tokens = vec![];
                            for t in self.connections.iter() {
                                if t.token.as_usize() != 0 {
                                    tokens.push(t.token);
                                }
                            }
                            for token in tokens {
                               self.connections[token].write_text_out(4,&msg); 
                               self.connections[token].reregister_writable(event_loop);
                            }
                        },
                        _ => {
                            panic!("Oh no!");
                        }
                    }
                },
                Err(_) => {
                    break; 
                },
            }
        }
        //Essentially this is acting as a coroutine to yield so other messages can be handled. 
        let _ = event_loop.timeout_ms(TIMEOUT, 1);
    }

    fn ready(&mut self, event_loop: &mut mio::EventLoop<Server>, token: mio::Token, events: mio::EventSet){
        match token {
            //If this connection comes from the server, that means it is a new connection being
            //opened
            SERVER => {
                match self.server.accept() {
                    Ok(Some(socket)) => {
                        let game = self.games.clone();
                        let token = self.connections
                        .insert_with(|token| Connection::new(game, socket, token))
                        .unwrap();
                        
                        event_loop.register_opt(&self.connections[token].socket,
                            token,
                            mio::EventSet::readable(),
                            mio::PollOpt::edge() | mio::PollOpt::oneshot()).unwrap();
                    },
                    Ok(None) => {
                        println!("Server wasn't ready");
                    },
                    Err(_) => {
                        println!("Something def fucked up");
                        //event_loop.shutdown();
                    },
                }
            },
            _ => {
                //otherwise, call the server's ready connection.
                if events.is_hup() {
                    {
                        self.connections[token].quit(event_loop);
                    }
                    let _ = self.connections.remove(token);
                } else {
                    self.connections[token].ready(event_loop);
                    //if the connection has closed, remove it
                    if self.connections[token].is_closed() {
                        let _ = self.connections.remove(token);
                    }
                }
            },
        }
    }
}

/// The connection implements some RPC protocol, as well as interfaces with the client. 
///
/// It basically keeps two queues. One for input and one for output. The input one is done anytime
/// something comes from the user. The output comes from the notify method of the server. This can
/// be triggered at any time, but when something is added to it, it will trigger a write out to the
/// client.
///
/// It holds a reference to the game, so that it can grab a game loop and such. 
///
/// It also holds the player I guess. I don't actually think that will be the case in later
/// versions. Player will be something created when the user connects. (Probably based on the class
/// they select.)
struct Connection {
    games: Arc<RefCell<Game>>,
    name: String,
    skin: String,
    socket: TcpStream,
    token: mio::Token,
    to_client_queue: Vec<ByteBuf>,
    event_set: mio::EventSet,
    state: State,
}

impl Connection{
    fn new(games: Arc<RefCell<Game>>, socket: TcpStream, token: mio::Token) -> Connection {
        Connection {
            games: games,
            socket: socket,
            name: "".to_string(),
            skin: "".to_string(),
            token: token,
            to_client_queue: vec![],
            event_set: mio::EventSet::readable(),
            state: State::NotLoggedIn,
        }
    }

    fn is_closed(&self) -> bool {
        match self.state {
            State::Closed => {
                true
            },
            _ => {
                false
            },
        }
    }

    fn quit(&mut self, event_loop: &mut mio::EventLoop<Server>) {
        println!("Quit parse");
        match self.games.borrow_mut().get_or_create_game_loop("maps/main.map") {
            Some(game_loop) => {
                game_loop.borrow_mut().remove(self.token.clone());
            },
            None => {},
        }
    }
    
    fn ready(&mut self, event_loop: &mut mio::EventLoop<Server>){
        //If readable && not logged in, send it to login
        // elif readable && logged in send to command reader (which will send to game_loop or chat)
        // if writable send to client writer
        match self.state {
            State::NotLoggedIn => {
                if self.event_set.is_readable() {
                    self.login(event_loop);
                }
            },
            State::LoggedIn => {
                if self.event_set.is_writable() {
                    self.event_set.remove(mio::EventSet::writable());
                    self.writable(event_loop);
                } else if self.event_set.is_readable() {
                    self.event_set.remove(mio::EventSet::readable());
                    self.readable(event_loop);
                }
            },
            _ => {
                unimplemented!();
            }
        }
    }
    
    fn readable(&mut self, event_loop: &mut mio::EventLoop<Server>) {
        let mut read = Vec::with_capacity(4096);
        match self.socket.try_read_buf(&mut read) {
            Ok(Some(0)) => {
                self.reregister_readable(event_loop);
            },
            Ok(Some(mut n)) => {
                //println!("Read {}", n);
                //Read strings. Write to game_loop
                loop {
                    if n >= 3 {
                        //Must be over 3 in length. 
                        let length_slice = &read[..2];
                        let length = length_slice.iter().fold(0usize,| total, x | total  << 8 | *x as
                                                             usize);
                        if n >= 2+ length {
                            let command = std::str::from_utf8(&read[2..2+length]).unwrap();
                            if command.starts_with("#tile") {
                                match command.split(" ").next().unwrap().parse() {
                                    Ok(tile) => {
                                        self.write_tile(tile);
                                    },
                                    _ => {},
                                }
                            } else if command.starts_with("#skin ") && command.len() > 5 {
                                match command.split(" ").next().unwrap().parse() {
                                    Ok(skin) => {
                                        self.skin = skin;
                                        match self.games.borrow_mut().get_or_create_game_loop("maps/main.map") {
                                            Some(game_loop) => {
                                                game_loop.borrow_mut().send_command(Msg::Command(self.token.clone(),
                                                command.to_string()));
                                            },
                                            None => {},
                                        }

                                    },
                                    _ => {},
                                }


                            } else if command.starts_with("shout ") && command.len() > 6 {

                                let (ref a, ref msg) = command.split_at(6);
                                let mut m = format!("{} shouts: {} ", self.name, msg).to_string();
                                //Doing this the trivially easy way, just doing a notification for
                                //that gets pushed to everyone
                                let send = self.games.borrow_mut().send.clone();
                                send.send(Msg::Shout(m));
                            } else if command.starts_with("join ") && command.len() > 5 {
                                let (ref a, ref msg) = command.split_at(6);
                                //Check to see if the server exists
                                //If map files exists it needs to create it.
                                //Remove player from the old map. Add to the new.
                                 
                            } else {
                                //println!("Readable parse");
                                match self.games.borrow_mut().get_or_create_game_loop("maps/main.map") {
                                    Some(game_loop) => {
                                        game_loop.borrow_mut().send_command(Msg::Command(self.token.clone(),
                                        command.to_string()));
                                    },
                                    None => {},
                                }
                            }
                            n = n - (2 + length);
                        }
                    } else {
                        break;
                    }
                }
                self.reregister_readable(event_loop);
            },
            Ok(None) => {
                self.reregister_readable(event_loop);
            },
            Err(_) => {
                panic!("Error reading");
            }
        }
    }
    
    fn writable(&mut self, event_loop: &mut mio::EventLoop<Server>) {
        let mut buf = self.to_client_queue.pop().unwrap();
        match self.socket.try_write_buf(&mut buf) {
            Ok(Some(n)) => {
                if buf.has_remaining() {
                    self.to_client_queue.push(buf);
                    self.event_set.insert(mio::EventSet::writable());
                }
                //println!("Wrote {} bytes", n);
                if self.to_client_queue.len() >  0 {
                    self.event_set.insert(mio::EventSet::writable());
                }
                self.reregister_readable(event_loop); 
                
            },
            Ok(None) => {
                //println!("Wrote to queue");
                self.to_client_queue.push(buf);
                self.reregister_writable(event_loop);
            },
            Err(_) => {
                panic!("Write error");
            }
        }
        if self.to_client_queue.len() > 0  {
            self.reregister_writable(event_loop);
        }
    }
    
    fn login(&mut self, event_loop: &mut mio::EventLoop<Server>) {
        //Do Login stuff
        let mut buf:Vec<u8> = Vec::with_capacity(150);
        match self.socket.try_read_buf(&mut buf) {
            Ok(Some(0)) => {
                        self.write_conn_result(4);
                        self.write_quit();
            },
            Ok(Some(n)) => {
                //Login
                //1 int (4), 2 short (2x2), 3 utf (3x3) = 17
                if n >= 17 {
                    //Modify this to use take.
                    let first_value: i32 = buf[..].iter().take(4).fold(0i32, |sum, x| sum  << 8 | *x as i32);
                    if first_value == 1 {
                        let width: i16 = buf[4..].iter().take(2).fold(0i16, |sum, x| sum << 8 | *x as i16);
                        //println!("width {}", width);
                        let height: i16 = buf[6..].iter().take(2).fold(0i16, |sum, x| sum << 8 | *x as
                                                                  i16);
                        //println!("height {}", height);
                        let name_len: usize= buf[8..].iter().take(2).fold(0usize, |sum, x| sum << 8 | *x as usize);
                        let name: &str = std::str::from_utf8(&buf[10..10+name_len]).unwrap();
                        self.name = name.to_string();
                        //println!("name {}", name);
                        let pw_len: usize= buf[10+name_len..].iter().take(2).fold(0usize, |sum, x| sum << 8 | *x as usize);
                        let pw: &str = std::str::from_utf8(&buf[12+name_len..12+name_len+pw_len]).unwrap();
                        //println!("pw {}", pw);
                        let version_len: usize= buf[12+name_len+pw_len..].iter().take(2).fold(0usize, |sum, x| sum << 8 | *x as usize);
                        //println!("Remanining {} Version {}", 16+name_len+pw_len, version_len);
                        //Hoping that it can't double send. Cause that'll produce weird results
                        let version: &str =
                            std::str::from_utf8(&buf[14+name_len+pw_len..]).unwrap();
                        //println!("version {}", version);
                        //println!("game_loop");
                        //Change to state logged in
                        self.state = State::LoggedIn;
                        //println!("state");
                        //Write to user they are logged in
                        //TODO Make writers optional, cause this needs to have registered writable
                        self.write_conn_result(3);
                        //println!("write_state");
                        //Send tile mappings for artwork
                        self.write_tile_mappings();
                        println!("Adding items");
                        if self.name.starts_with("paladin") {
                            //Writing paladin specific stats
                            self.write_stat_name("Rizato the Paladin");
                            self.write_inv_add("Shield Of Reflection","sell", "armor/shield/shield_of_reflection", 0,0);
                            self.write_inv_add("Ring of Protection","sell", "magic/ring_emerald", 0,0);
                            self.write_inv_add("Crystal Platemail","sell", "armor/armor/crystal_platemail.1", 0,0);
                            //Writing pally spells
                            self.write_ground_add("R - Full Heal", "cast", "magic/reagent/clover", 0,0);
                            self.write_ground_add("E - Holy Wrath", "cast", "spells/holy_wrath.1", 0, 0);
                            self.write_ground_add("W - Flame Sword", "cast", "weapons/artifact/flaming_sword.1", 0, 0);
                            self.write_ground_add("Q - Magic Shield","cast", "armor/shield/magical_shield", 0,0);
                        } else if self.name.starts_with("mage") {
                            self.write_stat_name("Cama the Arch Mage");
                            self.write_inv_add("Shirt of Fire","sell", "armor/armor/cloth_fire", 0,0);
                            self.write_inv_add("Ring of Fire","sell", "magic/ring_ruby", 0,0);
                            self.write_inv_add("Samhoc Crown","sell", "armor/helmet/samhoc_crown", 0,0);
                            self.write_inv_add("Wand of Power","sell", "magic/long_wand", 0,0);
                            //Writing pally spells
                            self.write_ground_add("R - Fireball", "cast", "spells/fireball/fireball_meteor.SE1", 0,0);
                            self.write_ground_add("E - Prismatic Shield", "cast", "spells/fireice.1", 0, 0);
                            self.write_ground_add("W - Curse", "cast", "spells/curse.1", 0, 0);
                            self.write_ground_add("Q - Fear","cast", "spells/fear.1", 0,0);
                        } else if self.name.starts_with("panther_male") {
                            self.write_stat_name("Romin the Warrior");
                            self.write_inv_add("Claws","sell", "weapons/claws/knop/claws_hunter", 0,0);
                            self.write_inv_add("Black Dragon Mail","sell", "armor/armor/black_dragon_mail", 0,0);
                            self.write_inv_add("Lion Cloak","sell", "armor/cloak/lion_cloak", 0,0);
                            //Writing pally spells
                            self.write_ground_add("R - Triple Strike", "cast", "spells/death.1", 0,0);
                            self.write_ground_add("E - Swipe & Poison", "cast", "spells/poison/poison.1", 0, 0);
                            self.write_ground_add("W - Reveal", "cast", "spells/eyeball", 0, 0);
                            self.write_ground_add("Q - Fog","cast", "spells/fog/fog", 0,0);
                        }
                        
                        self.write_stat_gold(123456);
                        self.write_stat_level(32, 8765534);
                        self.write_stat_all(200, 200, 100, 100, 25, 1000000, 3000000, 6, 10);
                        
                        println!("Tiles");
                        self.reregister_writable(event_loop);
                        println!("Writable");
                        println!("Login parse");
                        match self.games.borrow_mut().get_or_create_game_loop("maps/main.map") {
                            Some(game_loop) => {
                                game_loop.borrow_mut().join(self.token.clone(), self.name.clone());
                                println!("Looped");
                            },
                            None => {},
                        }
                        //This is here only while it is a single user. Normally, these would be added to the game_loop, not set.
                        self.reregister_readable(event_loop);
                    } else {
                        self.write_conn_result(4);
                        self.write_quit();
                    }
                }else {
                    self.write_conn_result(4);
                    self.write_quit();
                }
                //Need to find a way
            },
            Ok(None) => {
                self.reregister_readable(event_loop);
            },
            Err(_) => {
                println!("Ack! Error");
                //event_loop.shutdown();
            }
        };
        //If success...
        //get loop from server
    }
    
    fn reregister_writable(&mut self, event_loop: &mut mio::EventLoop<Server>){
        self.event_set.insert(mio::EventSet::writable());
        let _ = event_loop.reregister(&self.socket, self.token, self.event_set, mio::PollOpt::oneshot());
    }
    fn reregister_readable(&mut self, event_loop: &mut mio::EventLoop<Server>){
        self.event_set.insert(mio::EventSet::readable());
        let _ = event_loop.reregister(&self.socket, self.token, self.event_set, mio::PollOpt::oneshot());
    }
}

///Just implements the Api trait. This implementation is created from reading the web page that was
///posted with all this information.
impl Api for Connection {
    fn write_header(vec: &mut Vec<u8>, code:u8, length:i32) {
        vec.push(code);
        let high = length & 0xff0000;
        vec.push((high >> 16) as u8);
        let mid = length & 0xff00;
        vec.push((mid >> 8)  as u8);
        let low = length & 0xff;
        vec.push(low as u8);
    }

    fn write_string(vec: &mut Vec<u8>, string: &str) {
        let high= string.len() & 0xff00;
        vec.push((high >> 8) as u8);
        let low = string.len() & 0xff;
        vec.push(low as u8);
        vec.extend(string.as_bytes());
    }

    fn write_i16(vec: &mut Vec<u8>, short: i16) {
        let high= short as u16 & 0xff00;
        vec.push((high >> 8) as u8);
        let low = short & 0xff;
        vec.push(low as u8);
    }

    fn write_i16_reversed(vec: &mut Vec<u8>, short: i16) {
        let low = short & 0xff;
        vec.push(low as u8);
        let high= short as u16 & 0xff00;
        vec.push((high >> 8) as u8);
    }

    fn write_i32(vec: &mut Vec<u8>, val: i32) {
        let high = val as u32  & 0xff000000;
        vec.push((high >> 24) as u8);
        let mid_high = val & 0xff0000;
        vec.push((mid_high >> 16) as u8);
        let mid_low = val & 0xff00;
        vec.push((mid_low >> 8) as u8);
        let low = val & 0xff;
        vec.push(low as u8);
    }

    fn write_i32_reversed(vec: &mut Vec<u8>, val: i32) {
        let low = val & 0xff;
        vec.push(low as u8);
        let mid_low = val & 0xff00;
        vec.push((mid_low >> 8) as u8);
        let mid_high = val & 0xff0000;
        vec.push((mid_high >> 16) as u8);
        let high = val as u32  & 0xff000000;
        vec.push((high >> 24) as u8);
    }

    fn write_i64(vec: &mut Vec<u8>, val: i64) {
        let very_high = val as u64 & 0xff00000000000000;
        vec.push((very_high >> 56) as u8);
        let very_mid_high = val & 0xff000000000000;
        vec.push((very_mid_high >> 48) as u8);
        let very_mid_low = val &    0xff0000000000;
        vec.push((very_mid_low >> 40) as u8);
        let very_low = val &          0xff00000000;
        vec.push((very_low >> 32 )as u8);
        let high = val                & 0xff000000;
        vec.push((high >> 24) as u8);
        let mid_high = val & 0xff0000;
        vec.push((mid_high >> 16) as u8);
        let mid_low = val & 0xff00;
        vec.push((mid_low >> 8) as u8);
        let low = val & 0xff;
        vec.push(low as u8);
    }

    fn write_timestamp(vec: &mut Vec<u8>) {
        //println!("Timestamp");
        let current = time::get_time();
        let mut milli = current.sec * 1000;
        milli = milli + (current.nsec as i64 / 1000000);
        Connection::write_i64(vec, milli);
    }

    fn write_conn_result(&mut self, result: u8) {
        //println!("Conn Result");
        let mut conn: Vec<u8> = vec![];
        //connection result indication
        Connection::write_header(&mut conn, 2, 5);
        //major version
        Connection::write_i16(&mut conn, 1);
        //minor version
        Connection::write_i16(&mut conn, 1);
        //result
        conn.push(result);
        //So, ints, and the utf size are done in diferent directions.
        //conn.push(0);
        //conn.push(6);
        //conn.extend("Failed".as_bytes());
        self.to_client_queue.insert(0, ByteBuf::from_slice(&conn[..]));
    //  self.reregister_writable(event_loop);
    }
    
    fn write_quit(&mut self) {
        //println!("Quit");
        //quit indication
        let mut q: Vec<u8> = vec![];
        Connection::write_header(&mut q, 13, 0);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&q[..]));
    }
    
    fn write_tile_mappings(&mut self) {
        //println!("Mappings");
        //Read map files. Write out entire output.
        let mut uncompressed: Vec<u8> = vec![];
        for (k, v) in self.games.borrow().mappings.iter() {
           Connection::write_i16(&mut uncompressed, v.clone());
           Connection::write_string(&mut uncompressed, k.trim());
        }
        let u_len = uncompressed.len();
        let mut compressed = Connection::zip_data(uncompressed);
        let z_len = compressed.len();
        let total = z_len + 8;
        let mut tile: Vec<u8> = vec![];
        Connection::write_header(&mut tile, 8, total as i32);
        //zipped len
        Connection::write_i32(&mut tile, z_len as i32);
        //ulen
        Connection::write_i32(&mut tile, u_len as i32);
        //zipped contents
        tile.append(&mut compressed);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&tile[..]));
    }

    fn write_tile(&mut self, tile: i16) {
        //println!("Write tile");
        let mut path = String::new();
        for (k, v) in self.games.borrow().mappings.iter() {
            if v.clone() == tile {
                path = k.clone();
            }
        }
        let mut buf: Vec<u8> = vec![];
        Connection::write_header(&mut buf, 16, 12 + path.len() as i32);
        Connection::write_i16(&mut buf, tile);
        Connection::write_string(&mut buf, &path);
        Connection::write_timestamp(&mut buf);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&buf[..]));
    }
    
    fn write_text_out(&mut self, style: u8, message: &str) {
        //println!("Text Out");
        let mut m: Vec<u8> = vec![];
        //text out byte
        Connection::write_header(&mut m, 11, (message.len()+3) as i32);
        m.push(style);
        Connection::write_string(&mut m, message); 
        self.to_client_queue.insert(0, ByteBuf::from_slice(&m[..]));
        //self.reregister_writable(event_loop);
    }
    
    fn write_zipped_screen(&mut self, screen: MapScreen) {
        //Convert MapScreen to zipped screen.
        let mut uncompressed: Vec<u8> = vec![];
        for terrain in screen.terrain.iter() {
            match self.games.borrow_mut().mappings.get(&terrain.tile) {
                Some(tile) => {
                    Connection::write_i32_reversed(&mut uncompressed, tile.clone() as i32);
                },
                None => {
                    //Writing an invalid tile so it just shows the dot pattern
                    Connection::write_i32_reversed(&mut uncompressed, 9999);
                },
            }
        }
        //println!("Zipped screen");
        for object in screen.objects.iter() {
            //println!("Object! {}", object.tile);
            uncompressed.push(object.y);
            uncompressed.push(object.x);
            match self.games.borrow_mut().mappings.get(&object.tile) {
                Some(tile) => {
                    Connection::write_i16_reversed(&mut uncompressed, tile.clone() as i16);
                },
                None => {
                    //Writing an invalid tile so it just shows the dot pattern
                    Connection::write_i16_reversed(&mut uncompressed, 9999);
                },
            }
        }
        //println!("Zipped screen");
        let u_len = uncompressed.len();
        let mut compressed = Connection::zip_data(uncompressed);
        let z_len = compressed.len();
        let total = 12 + z_len;
        let mut s : Vec<u8> = vec![];
        Connection::write_header(&mut s, 24, total as i32);
        //width
        Connection::write_i16(&mut s,15); 
        //height
        Connection::write_i16(&mut s,15); 
        //zipped len
        Connection::write_i32(&mut s,z_len as i32); 
        //ulen
        Connection::write_i32(&mut s,u_len as i32); 
        //zipped contents
        s.append(&mut compressed);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&s[..]));
        //println!("Zipped screen");
    }

    fn write_stat_name(&mut self, name: &str) {
        //println!("Name");
        let mut n = vec![];
        Connection::write_header(&mut n, 106, 2 + name.len() as i32);
        Connection::write_string(&mut n, name);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&n[..]));
    }

    fn write_stat_gold(&mut self, gold: i32) {
        //println!("Write gold");
        let mut n = vec![];
        Connection::write_header(&mut n, 104, 4);
        Connection::write_i32(&mut n, gold);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&n[..]));
    }

    fn write_stat_level(&mut self, level: u8, xp: i32) {
        //println!("Write level");
        let mut n = vec![];
        Connection::write_header(&mut n, 105, 5);
        n.push(level);
        Connection::write_i32(&mut n, xp);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&n[..]));
    }

    fn write_stat_all(&mut self, hp: i32, mhp: i32, sp: i32, msp: i32, level: i32, xp: i32, nxp:
                      i32, food: i32, mfood: i32) {
        //println!("WRITE STATS");
        let mut n = vec![];
        Connection::write_header(&mut n, 120, 36 );
        Connection::write_i32(&mut n, hp);
        Connection::write_i32(&mut n, mhp);
        Connection::write_i32(&mut n, sp);
        Connection::write_i32(&mut n, msp);
        Connection::write_i32(&mut n, level);
        Connection::write_i32(&mut n, xp);
        Connection::write_i32(&mut n, nxp);
        Connection::write_i32(&mut n, food);
        Connection::write_i32(&mut n, mfood);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&n[..]));
        
    }
                      
    fn write_ground_add(&mut self, name: &str, commands: &str, tile_name: &str, index: i16, offsets: i16) {
        //println!("Write ground add {}", name);
        let mut n = vec![];
        Connection::write_header(&mut n, 80, (10 + name.len() + commands.len()) as i32);
        let tile = self.games.borrow_mut().mappings.get(tile_name).unwrap().clone();
        Connection::write_string(&mut n, name);
        Connection::write_string(&mut n, commands);
        Connection::write_i16(&mut n, tile);
        Connection::write_i16(&mut n, index);
        Connection::write_i16(&mut n, offsets);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&n[..]));
    }
    
    fn write_inv_add(&mut self, name: &str, commands: &str, tile_name: &str, index: i16, offsets: i16) {
        //println!("Write inv add {}", name);
        let mut n = vec![];
        Connection::write_header(&mut n, 70, (10 + name.len() + commands.len()) as i32);
        let tile = self.games.borrow_mut().mappings.get(tile_name).unwrap().clone();
        Connection::write_string(&mut n, name);
        Connection::write_string(&mut n, commands);
        Connection::write_i16(&mut n, tile);
        Connection::write_i16(&mut n, index);
        Connection::write_i16(&mut n, offsets);
        self.to_client_queue.insert(0, ByteBuf::from_slice(&n[..]));
    }
    
    fn zip_data(data: Vec<u8> ) -> Vec<u8> {
        //println!("ZIP");
        let d = data.len();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::Default);
        let mut written = 0;
        while written < d {
            written = written + encoder.write(&data[written..]).unwrap();
        }
        encoder.finish().unwrap()
    }
}
