/*
  Copyright 2015 Robert Lathrop

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

use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::sync::RwLock;
use std::sync::Arc;

use game::gamemap::GameMap;
use game::characters::player::Player;
use conn::server::Msg;

pub struct GameLoop {
	//Map with all items & tiles
	game_map: Arc<RwLock<GameMap>>,
    connections: Arc<RwLock<Vec<mio::Token>>>, 
    send: mio::Sender<Msg>,
}

impl GameLoop {
	pub fn new(mapname : &str, send: mio::Sender<Msg>) -> GameLoop {
		GameLoop {
			game_map: Arc::new(RwLock::new(GameMap::new(mapname).unwrap())),
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
               thread::sleep(Duration::from_millis(20));
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
               let mut map = game_map.write().unwrap();
               //This can cause DOS by keeping the commands from executing
               'outer: loop {
                   match recv.try_recv() {
                       Ok(Msg::Command(token, command)) => {
                           //println!("{}", command);
                           &map.push_command(token, command); 
                       },
                       _ => {
                           //println!("Nothin.");
                           break 'outer; 
                       }
                   }
               }
               //TODO get these responses in there somehow
               let responses = map.execute(&mutex);
               //Cannot seem to decontruct tuples in a loop. Doing the index version instead of
               //iterating
               for i in 0..responses.len() {
                   let (token, style, response) = responses[i].clone();
               	   to_mio.send(Msg::TextOutput(token, style, response));
               }
               //send map & health updates
               for conn in mutex.iter() {
                   let screen = map.send_portion(conn.clone());
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
    
    pub fn join(&mut self, token: mio::Token, player: Arc<Player>) {
        let mut conn = self.connections.write().unwrap();
        self.game_map.write().unwrap().add_player(token.clone(), player);
        conn.push(token);
    }
}
