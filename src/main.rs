extern crate mio;
extern crate flate2;
extern crate byteorder;
extern crate time;

use mio::{TryRead, TryWrite};
use mio::tcp::*;
use mio::util::Slab;
use mio::buf::ByteBuf;
use mio::buf::Buf;
use flate2::Compression;
use flate2::write::ZlibEncoder;

use std::io::prelude::*;
use std::io::BufReader;
use std::net::SocketAddr;
use std::fs::File;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::RwLock;

//Setting the server as the first token
const SERVER: mio::Token = mio::Token(0);

fn main() {
    println!("starting");
    let addr: SocketAddr = "0.0.0.0:2222".parse().unwrap();
    println!("addr");
    let server = TcpListener::bind(&addr).unwrap();
    println!("server");
    let mut event_loop = mio::EventLoop::new().unwrap();
    println!("event_loop"); 
    event_loop.register(&server, SERVER).unwrap();
    println!("register");

    let mut moba = Server::new(server);
    let _ = event_loop.run(&mut moba).unwrap();
}

//This reflects the structure of the network API.
#[derive(Clone)]
struct GameMap {
	//This
	terrain: Vec<MapTerrain>,
	//Items includes teleports, loot, traps, gold, etc
	items: Vec<MapItem>,
	//This is any players in the map
	users: Vec<MapUser>,
	//All monsters and other npcs
	npcs: Vec<MapNpc>,
}

impl GameMap {
	fn new(mapname: &str) -> Result<GameMap, &str> {
        //TODO Load map from file.
		let map = GameMap {
			terrain: vec![],
			items: vec![],
			users: vec![],
			npcs: vec![],
		};
		Ok(map)
	}

	fn execute(&self, command: String) -> &str {
        println!("{}",command);
		"Command executed "
	}
	
	fn execute_all(&self) {
		println!("Executed");
	}
	
	fn send_portion(&self) -> MapScreen {
		//This sends the squares around the user, which will always be centered in the screen.
		MapScreen::new()
	}
}

#[derive(Clone)]
struct MapTerrain {
	//No position, because position is determined by the position in vector
	tile: String,
}

impl MapTerrain {
	fn new(tile: String) -> MapTerrain {
		MapTerrain{
			tile: tile,
		}
	}
}

#[derive(Clone)]
struct MapItem {
	//item: Item,
	tile: i16,
	x: u8,
	y: u8,
}

#[derive(Clone)]
struct MapUser{
	//player: Player,
	tile: i16,
	x: u8,
	y: u8,
}

#[derive(Clone)]
struct MapNpc{
	//npc: Npc,
	tile: i16,
	x: u8,
	y: u8,
}

#[derive(Clone)]
struct MapObject {
	tile: String,
	x: u8,
	y: u8,
}

impl MapObject {
	fn new(tile: String, x: u8, y:u8) -> MapObject{
		MapObject{
			tile: tile,
			x: x,
			y: y,
		}
	}
}


#[derive(Clone)]
struct MapScreen {
	//15x15 vector. 
	terrain: Vec<MapTerrain>,
	//User art at 7,7
	objects: Vec<MapObject>,
}

impl MapScreen {
	fn new() -> MapScreen {
		let ter = vec![MapTerrain::new("terrain/wood_floor".to_string()); 225];
		let mut obj = vec![];
		obj.push(MapObject::new("players/wizard.S".to_string(), 7, 7));
		obj.push(MapObject::new("wiz/binyamin/art/greatshoggoth.E".to_string(), 3, 4));
		MapScreen {
			terrain: ter,
			objects:obj,
		}
	}
}

struct GameLoop {
	//Map with all items & tiles
	game_map: Arc<GameMap>,
    connections: Arc<RwLock<Vec<mio::Token>>>, 
    send: mio::Sender<Msg>,
}

impl GameLoop {
	fn new(mapname : &str, send: mio::Sender<Msg>) -> GameLoop {
		GameLoop {
			game_map: Arc::new(GameMap::new(mapname).unwrap()),
			connections: Arc::new(RwLock::new(vec![])),
            send: send,
		}
	}
	
	
	pub fn start(&mut self) {
        let game_map = self.game_map.clone();
        let connections = self.connections.clone();
        let to_mio = self.send.clone();
		thread::spawn(move || {
           let (send, recv) = channel(); 
           loop {
               let mut threads = vec![];
               thread::sleep(Duration::from_millis(1000));
               //let screen_out = screen.clone();
               //to_mio.send(Msg::Screen(mio::Token(1), screen_out));
               let mutex = connections.read().unwrap();
               for connection in mutex.iter(){ 
                   let s = send.clone();
                   let c = connection.clone();
                   let t = to_mio.clone();
                   threads.push(thread::spawn(move|| {
                        let _ = t.send(Msg::SendCommand(c, s));
                   }));
               }
               for t in threads {
                    t.join().unwrap();
               }
               let mut commands: Vec<(mio::Token, String)> = vec![];
               //This can cause DOS by keeping the commands from executing
               'outer: loop {
                   match recv.try_recv() {
                       Ok(Msg::Command(token, command)) => {
                           println!("{}", command);
                           &commands.push((token, command)); 
                       },
                       _ => {
                           println!("Nothin.");
                           break 'outer; 
                       }
                   }
               }
               for (token, comm) in commands {
                    let response =  game_map.execute(comm);
                    //send command execution response (Use this to send item/health updates from recoil)
                    let _ = to_mio.send(Msg::TextOutput(token, 2, response.to_string()));
               }
               game_map.execute_all();
               //send map & health updates
               //his map should be based on the player position normally.
               let screen = game_map.send_portion();
               for conn in mutex.iter() {
                   //Need to see response from sender
                   match to_mio.send(Msg::Screen(conn.clone(), screen.clone())) {
                        Err(mio::NotifyError::Io(_)) => {
                            println!("IO");
                        },
                        Err(mio::NotifyError::Full(_)) => {
                            println!("FUll");
                        },
                        Err(mio::NotifyError::Closed(_)) => {
                            println!("Closed");
                        },
                        Ok(_) => {
                        },
                   }
               }
           }
        });
	}
    
    fn join(&mut self, token: mio::Token) {
        self.connections.write().unwrap().push(token);
    }
}

struct Game {
	game_loops: HashMap<String, Arc<RefCell<GameLoop>>>,
    mappings: HashMap<String, i16>,
}

impl Game {
    fn new() -> Game {
        let mut m: HashMap<String,i16> = HashMap::new();  
		let tile_file = File::open("file_full").unwrap(); 
		let mut reader = BufReader::new(tile_file);
		let mut line: String = String::new();
        let mut count = 0;
        while reader.read_line(&mut line).unwrap() > 0 {
            m.insert(line.clone().trim().to_string(), count.clone());
            count = count + 1;
            line.clear();
		}
        Game {
            game_loops: HashMap::new(),
            mappings: m,
        }
    }

	fn get_or_create_game_loop(&mut self, map_name: &str, event_loop: &mut mio::EventLoop<Server>) -> Arc<RefCell<GameLoop>> {
        println!("{}", map_name);
		//This can handle all kinds of things. Checks last time user was inside, if too long it recreates. 
		//Checks the hashmap for the Gameloop. If not there, it creates a new one, adds it and returns it.
        let send = event_loop.channel();
        let _ = send.send(Msg::TextOutput(mio::Token(1), 2, "test".to_string()));
        let game_loop = Arc::new(RefCell::new(GameLoop::new(map_name, send)));
        game_loop.borrow_mut().start();
        self.game_loops.insert(map_name.to_string(), game_loop.clone());
        game_loop
	}
}

struct Server {
	server: TcpListener,
	//Tried removing the Arc here
	connections: Slab<Connection>,
    games: Arc<RefCell<Game>>,
}

impl Server {
	fn new(tcp: TcpListener) -> Server {
		let slab = Slab::new_starting_at(mio::Token(1), 1024);
		Server {
			server: tcp,
			connections: slab,
            games: Arc::new(RefCell::new(Game::new())),
		}
	}
}

impl mio::Handler for Server {
    type Timeout = ();
    type Message = Msg;

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Server>, msg: Self::Message) {
		match msg {
			Msg::TextOutput(token, result, message) => {
				// Write message
				self.connections[token].write_text_out(result, &message);
                self.connections[token].reregister_writable(event_loop);
			},
			Msg::Screen(token, screen) => {
				//Write screen
				self.connections[token].write_zipped_screen(screen);
                self.connections[token].reregister_writable(event_loop);
			},
            Msg::SendCommand(token, send) => {
                //Tell it to send a command
                self.connections[token].send_command(send);
            },
			_ => {
                panic!("Oh no!");
			}
		}
    }

	fn ready(&mut self, event_loop: &mut mio::EventLoop<Server>, token: mio::Token, events: mio::EventSet){
		match token {
            SERVER => {
                match self.server.accept() {
			        Ok(Some(socket)) => {
                        let game = self.games.clone();
			        	let token = self.connections
			        	//Removed conn as an Arc. We will see how that goes.
                            //Trying to do this with 
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
				self.connections[token].ready(event_loop);
				if self.connections[token].is_closed() {
					let _ = self.connections.remove(token);
				}
			},
        }
	}
}

enum State {
    Closed,
	NotLoggedIn,
	LoggedIn,
}

enum Msg {
	Command(mio::Token, String),
    SendCommand(mio::Token, Sender<Msg>),
    TextOutput(mio::Token, u8, String),
    Screen(mio::Token, MapScreen)
}

struct Player{
	hp: i32,
	max_hp: i32,
	name: String,
}

impl Player {
	fn new() -> Player {
		Player {
			hp: 0,
			max_hp: 0,
			name: "empty".to_string(), 
		}
	}
}

struct Connection {
    games: Arc<RefCell<Game>>,
	socket: TcpStream,
    player: Player,
	token: mio::Token,
	to_client_queue: Vec<ByteBuf>,
	from_client_queue: Vec<String>,
	event_set: mio::EventSet,
	state: State,
}

impl Connection{
	fn new(games: Arc<RefCell<Game>>, socket: TcpStream, token: mio::Token) -> Connection {
        let play = Player::new();
		Connection {
            games: games,
			socket: socket,
            player: play,
			token: token,
			to_client_queue: vec![],
			from_client_queue: vec![],
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
    
    fn send_command(&mut self, send: Sender<Msg>) {
    	if self.from_client_queue.len() > 0 {
    		let command = self.from_client_queue.pop().unwrap();
	    	send.send(Msg::Command(self.token.clone(), command));
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
		let mut read = vec![];
        println!("Readable");
		match self.socket.try_read_buf(&mut read) {
			Ok(Some(0)) => {
				self.reregister_readable(event_loop);
			},
			Ok(Some(mut n)) => {
                println!("Read {}", n);
				//Read strings. Write to game_loop
				loop {
					if n > 3 {
						//Must be over 3 in length. 
						let length_slice = &read[..2];
						let length = length_slice.iter().fold(0usize,| total, x | total  << 8 | *x as
                                                             usize);
						if n >= 2+ length {
							let command = std::str::from_utf8(&read[2..length]).unwrap();
                            if command.starts_with("#tile") {
                                let tile: i16 = command.split(" ").next().unwrap().parse().unwrap();
                                self.write_tile(tile);
                            } else {
							    self.from_client_queue.push(command.to_string());
                                println!("Connection: {}", command);
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
				println!("Wrote {} bytes", n);
				if self.to_client_queue.len() >  0 {
					self.event_set.insert(mio::EventSet::writable());
				}
				self.reregister_readable(event_loop); 
				
			},
			Ok(None) => {
                println!("Wrote to queue");
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
		let mut buf:Vec<u8> = vec![];
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
                        println!("width {}", width);
						let height: i16 = buf[6..].iter().take(2).fold(0i16, |sum, x| sum << 8 | *x as
                                                                  i16);
                        println!("height {}", height);
						let name_len: usize= buf[8..].iter().take(2).fold(0usize, |sum, x| sum << 8 | *x as usize);
						let name: &str = std::str::from_utf8(&buf[10..10+name_len]).unwrap();
                        println!("name {}", name);
						let pw_len: usize= buf[10+name_len..].iter().take(2).fold(0usize, |sum, x| sum << 8 | *x as usize);
						let pw: &str = std::str::from_utf8(&buf[12+name_len..12+name_len+pw_len]).unwrap();
                        println!("pw {}", pw);
						let version_len: usize= buf[12+name_len+pw_len..].iter().take(2).fold(0usize, |sum, x| sum << 8 | *x as usize);
                        println!("Remanining {} Version {}", 16+name_len+pw_len, version_len);
						let version: &str =
                            std::str::from_utf8(&buf[14+name_len+pw_len..14+name_len+pw_len+version_len]).unwrap();
                        println!("version {}", version);
                        println!("game_loop");
			            //Change to state logged in
			            self.state = State::LoggedIn;
                        println!("state");
			            //Write to user they are logged in
                        //TODO Make writers optional, cause this needs to have registered writable
			            self.write_conn_result(3);
                        println!("write_state");
			            //Send tile mappings for artwork
			            self.write_tile_mappings();
                        self.write_stat_name("Hunterz");
                        self.write_stat_gold(123456);
                        self.write_stat_level(32, 8765534);
                        self.write_stat_all(150, 200, 100, 100, 25, 1000000, 3000000, 6, 10);
                        println!("Tiles");
                        self.reregister_writable(event_loop);
                        println!("Writable");
                        let game_loop = self.games.borrow_mut().get_or_create_game_loop("map", event_loop);
			            game_loop.borrow_mut().join(self.token.clone());
                        println!("Looped");
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

impl WyvernApi for Connection {
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
        let current = time::get_time();
        let mut milli = current.sec * 1000;
        milli = milli + (current.nsec as i64 / 1000000);
        Connection::write_i64(vec, milli);
    }

	fn write_conn_result(&mut self, result: u8) {
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
	//	self.reregister_writable(event_loop);
	}
	
	fn write_quit(&mut self) {
		//quit indication
		let mut q: Vec<u8> = vec![];
        Connection::write_header(&mut q, 13, 0);
		self.to_client_queue.insert(0, ByteBuf::from_slice(&q[..]));
	}
	
	fn write_tile_mappings(&mut self) {
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
        //self.to_client_queue.insert(0, ByteBuf::from_slice(&buf[..]));
    }
	
	fn write_text_out(&mut self, style: u8, message: &str) {
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
            let tile = self.games.borrow_mut().mappings.get(&terrain.tile).unwrap().clone();
            Connection::write_i32_reversed(&mut uncompressed, tile.clone() as i32);
		}
		for object in screen.objects.iter() {
			uncompressed.push(object.x);
			uncompressed.push(object.y);
            let tile = self.games.borrow_mut().mappings.get(&object.tile).unwrap().clone();
            Connection::write_i16_reversed(&mut uncompressed, tile.clone());
		}
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
	}

    fn write_stat_name(&mut self, name: &str) {
        let mut n = vec![];
        Connection::write_header(&mut n, 106, 2 + name.len() as i32);
        Connection::write_string(&mut n, name);
		self.to_client_queue.insert(0, ByteBuf::from_slice(&n[..]));
    }

    fn write_stat_gold(&mut self, gold: i32) {
        let mut n = vec![];
        Connection::write_header(&mut n, 104, 4);
        Connection::write_i32(&mut n, gold);
		self.to_client_queue.insert(0, ByteBuf::from_slice(&n[..]));
    }

    fn write_stat_level(&mut self, level: u8, xp: i32) {
        let mut n = vec![];
        Connection::write_header(&mut n, 105, 5);
        n.push(level);
        Connection::write_i32(&mut n, xp);
		self.to_client_queue.insert(0, ByteBuf::from_slice(&n[..]));
    }

    fn write_stat_all(&mut self, hp: i32, mhp: i32, sp: i32, msp: i32, level: i32, xp: i32, nxp:
                      i32, food: i32, mfood: i32) {
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
	
	fn zip_data(data: Vec<u8> ) -> Vec<u8> {
        let d = data.len();
		let mut encoder = ZlibEncoder::new(Vec::new(), Compression::Default);
        let mut written = 0;
        while written < d {
		    written = written + encoder.write(&data[written..]).unwrap();
        }
		encoder.finish().unwrap()
	}
}

trait WyvernApi {
    fn write_header(vec: &mut Vec<u8>, code:u8, lenth:i32);
    fn write_string(vec: &mut Vec<u8>, string: &str);
    fn write_i16(vec: &mut Vec<u8>, short: i16);
    fn write_i16_reversed(vec: &mut Vec<u8>, short: i16);
    fn write_i32(vec: &mut Vec<u8>, val: i32);
    fn write_i32_reversed(vec: &mut Vec<u8>, val: i32);
    fn write_i64(vec: &mut Vec<u8>, val: i64);
    fn write_timestamp(vec: &mut Vec<u8>);
	fn write_conn_result(&mut self, result: u8);
	fn write_quit(&mut self);
	fn write_tile_mappings(&mut self);
	fn write_tile(&mut self, tile: i16);
	fn write_text_out(&mut self, style: u8, message: &str);
	fn write_zipped_screen(&mut self,screen: MapScreen);
    fn write_stat_name(&mut self, name: &str);
    fn write_stat_gold(&mut self, gold: i32);
    fn write_stat_level(&mut self,level: u8, xp: i32);
    fn write_stat_all(&mut self, hp: i32, mhp: i32, sp: i32, msp: i32, level: i32, xp: i32, nxp:
                      i32, food: i32, mfood: i32);
	fn zip_data(data: Vec<u8>) -> Vec<u8>;
}
