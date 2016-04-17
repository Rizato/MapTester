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
///After commands have run, it uses the event loop channel to notify mio of messages to send to the
///users.
///

extern crate mio;

use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::sync::RwLock;
use std::sync::Mutex;
use std::sync::Arc;

use game::gamemap::GameMap;
use game::characters::player::Player;
use conn::server::Msg;

/// This struct holds the map, and a list of tokens that are connected to this game loop. 
/// This structure creates the background thread that operates the actual game loop.
pub struct GameLoop {
    //Map with all items & tiles
    game_map: String,
    connections: Arc<RwLock<Vec<mio::Token>>>, 
    add_connections: Arc<RwLock<Vec<(mio::Token, String)>>>, 
    remove_connections: Arc<RwLock<Vec<mio::Token>>>, 
    command_queue: Arc<Mutex<Vec<Msg>>>, 
    to_game_send: Sender<Msg>,
}

impl GameLoop {
    ///creates a new game loop
    pub fn new(mapname : &str, send: Sender<Msg>) -> Option<GameLoop> {
        let map = GameMap::new(&mapname);
        match map {
            Err(s) => {
                println!("{}", s);
                None
            }, 
            Ok(map) => {
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
    
    
    ///Creates the game loop
    ///This loop sends a message to all connections to ask for new commands
    ///Then it listens on the created channel for any commands.
    ///Then it pushes the commands to the MapUser objects 
    ///After that, the map executes all of the commands.
    ///These actions create responses that are doled out accoridng to connection
    ///The loop then sends the responses, and updated states then updated maps
    ///
    /// TODO Redo the commands to reduce the amount sent over the notification channel.
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
                       {
                           let mut a = add.write().unwrap();
                           let mut conn = connections.write().unwrap();
                           for i in 0..a.len() {
                               let (t, name) = a[i].clone();
                               map.add_player(t, name);
                               a.remove(i);
                               conn.push(t);
                          }
                       }
                       {
                           let mut r = remove.write().unwrap();
                           let mut conn = connections.write().unwrap();
                           for i in 0..r.len() {
                               let t = r[i].clone();
                               r.remove(i);
                               map.remove_player(t);
                               for i in 0..conn.len() {
                                   if conn[i] == t{
                                       conn.remove(i);
                                       break;
                                   }
                               }
                          }
                       }
                       //This can cause DOS by keeping the commands from executing
                       //println!("Reading commands");
                       //Putting this in a scope so that the commands can be repopulated when it is executing.
                       {
                            let mut c = commands.lock().unwrap();
                            for m in c.drain(..) {
                                match m {
                                    Msg::Command(token, command) => {
                                        println!("{}", command);
                                        &map.push_command(token.clone(), command.clone()); 
                                    },
                                    _ => {
                                        //println!("Nothin.");
                                    },
                                }
                            }
                       }
                       //TODO get these responses in there somehow
                       let mutex = connections.read().unwrap();
                       let responses = map.execute(&mutex);
                       //Cannot seem to decontruct tuples in a loop. Doing the index version instead of
                       //iterating
                       //println!("Reading Responses");
                       for i in 0..responses.len() {
                           let (token, style, response) = responses[i].clone();
                           to_mio.send(Msg::TextOutput(token, style, response));
                       }
                       //send map & health updates
                       //println!("Sending map");
                       for conn in mutex.iter() {
                           let hp = map.get_hp(conn.clone());
                           if hp.is_some() {
                               to_mio.send(Msg::Hp(conn.clone(), hp.unwrap()));
                           }
                           let screen = map.send_portion(conn.clone());
                           //Need to see response from sender
                           match screen {
                               Some(s) => {
                                   to_mio.send(Msg::Screen(conn.clone(), s));
                               },
                               None => {},
                           }
                       }
                       //println!("Finished Loop");
                   }
               }, 
                   Err(_) => {},
           }
        });
    }
    
    ///Lets a connection join the game loop
    pub fn join(&mut self, token: mio::Token, name: String) {
        let mut conn = self.add_connections.write().unwrap();
        conn.push((token, name));
    }

    pub fn remove(&mut self, token : mio::Token) {
        let mut conn = self.remove_connections.write().unwrap();
        conn.push(token);
    }

    pub fn send_command(&mut self, message: Msg) {
        self.command_queue.lock().unwrap().push(message);
    }
}
