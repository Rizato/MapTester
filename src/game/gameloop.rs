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

///This module handles the game loop.
///
///It handle connections between the mio connections as well. 
///Basically, it just glues the connection commands to the map, which executes the map on the
///thread provided by the game loop.
///
///After commands have run, it uses a channel to notify mio of messages to send to the
///clients.
extern crate mio;

use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::sync::RwLock;
use std::sync::Mutex;
use std::sync::Arc;

use game::gamemap::GameMap;
use conn::server::Msg;

/// This struct holds the map name, and a list of tokens that are connected to this game loop. 
/// This structure creates the background thread that operates the actual game loop.
pub struct GameLoop {
    //Map with all items & tiles
    game_map: String,
    connections: Arc<RwLock<Vec<mio::Token>>>, 
    add_connections: Arc<RwLock<Vec<(mio::Token, String, Option<(u8,u8)>)>>>, 
    remove_connections: Arc<RwLock<Vec<mio::Token>>>, 
    command_queue: Arc<Mutex<Vec<Msg>>>, 
    to_game_send: Sender<Msg>,
}

impl GameLoop {
    ///creates a new game loop
    pub fn new(mapname : &str, send: Sender<Msg>) -> Option<GameLoop> {
        if mapname.contains("..") {
            println!("Attempted relative path: {}", mapname);
            None
        } else {
            let map = GameMap::new(&mapname);
            match map {
                Err(s) => {
                    println!("{}", s);
                    None
                }, 
                Ok(_) => {
                    let mut gloop = GameLoop {
                        game_map: mapname.to_string(),
                        connections: Arc::new(RwLock::new(vec![])),
                        add_connections: Arc::new(RwLock::new(vec![])),
                        remove_connections: Arc::new(RwLock::new(vec![])),
                        command_queue: Arc::new(Mutex::new(vec![])),
                        to_game_send: send,
                    };
                    gloop.start();
                    Some(gloop)
                }
            }
        }
    }
    
    
    ///Creates the game loop
    ///This will read any incoming commands, send them to the map for execution,
    ///then relay the results to the clients.
    pub fn start(&mut self) {
        let connections = self.connections.clone();
        let add = self.add_connections.clone();
        let remove = self.remove_connections.clone();
        let commands = self.command_queue.clone();
        let to_mio = self.to_game_send.clone();
        let map = self.game_map.clone();
        thread::spawn(move || {
           let game_map = GameMap::new(&map);
           match game_map {
               Ok(mut map) => {
                   loop {
                       thread::sleep(Duration::from_millis(20));
                       //Have to do this inside a custom scope so the mutex will release
                       {
                           let mut a = add.write().unwrap();
                           let mut conn = connections.write().unwrap();
                           for i in 0..a.len() {
                               let mut exists = false;
                               let (t, name, index) = a[i].clone();
                               for c in 0..conn.len() {
                                   if t.as_usize() == conn[c].as_usize() {
                                       exists = true;
                                       break;
                                   }
                               }
                               if !exists {
                                   map.add_player(t, name, index);
                                   conn.push(t);
                               }
                           }
                           a.clear();
                       }
                       //Have to do this inside a custom scope so the mutex will release
                       {
                           let mut r = remove.write().unwrap();
                           let mut conn = connections.write().unwrap();
                           for i in 0..r.len() {
                               let t = r[i].clone();
                               map.remove_player(t);
                               for c in 0..conn.len() {
                                   if conn[c] == t{
                                       conn.remove(i);
                                       break;
                                   }
                               }
                           }
                           r.clear();
                       }
                       //Putting this in a scope so that the commands can be repopulated when it is executing other parts
                       {
                            let mut c = commands.lock().unwrap();
                            for m in c.drain(..) {
                                match m {
                                    Msg::Command(token, command) => {
                                        println!("{}", command);
                                        &map.push_command(token.clone(), command.clone()); 
                                    },
                                    _ => {},
                                }
                            }
                       }
                       //Execute map
                       {
                           let responses = map.execute();
                           //Cannot seem to decontruct tuples in a loop. Doing the index version instead of
                           //iterating
                           for i in 0..responses.len() {
                               let (token, style, response) = responses[i].clone();
                               let _ = to_mio.send(Msg::TextOutput(token, style, response));
                           }
                           //send map & health updates
                           let mutex = connections.read().unwrap();
                           for conn in mutex.iter() {
                               let hp = map.get_hp(conn.clone());
                               if hp.is_some() {
                                   let _ = to_mio.send(Msg::Hp(conn.clone(), hp.unwrap()));
                               }
                               let screen = map.send_portion(conn.clone());
                               match screen {
                                   Some(s) => {
                                       let _ =to_mio.send(Msg::Screen(conn.clone(), s));
                                   },
                                   None => {},
                               }
                           }
                       }
                       //This handles any teleportations. It basically just looks at all users,
                       //if they are on a teleporter it sends a Join message, and removes them from
                       //this loop & its map.
                       let teleports = map.do_teleports();
                       for i in 0..teleports.len() {
                           let (token, join, index) = teleports[i].clone();
                           map.remove_player(token);
                           let mut conn = connections.write().unwrap();
                           for i in 0..conn.len() {
                               if conn[i] == token{
                                   conn.remove(i);
                                   break;
                               }
                           }
                           let _ = to_mio.send(Msg::Join(token, join, index));
                       }
                   }
               }, 
                   Err(_) => {},
           }
        });
    }
    
    ///Adds a token to be added
    pub fn join(&mut self, token: mio::Token, name: String, index: Option<(u8, u8)>) {
        let mut conn = self.add_connections.write().unwrap();
        conn.push((token, name, index));
    }

    ///Adds a token to be removed.
    pub fn remove(&mut self, token : mio::Token) {
        let mut conn = self.remove_connections.write().unwrap();
        conn.push(token);
    }

    ///Passes a command to the game loop
    pub fn send_command(&mut self, message: Msg) {
        self.command_queue.lock().unwrap().push(message);
    }
}