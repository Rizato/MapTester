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

pub mod gameloop;
pub mod gamemap;
pub mod characters;


use std::io::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;

use game::gameloop::GameLoop;
use conn::server::{Server, Msg};


pub struct Game {
	game_loops: HashMap<String, Arc<RefCell<GameLoop>>>,
    pub mappings: HashMap<String, i16>,
}

impl Game {
    pub fn new() -> Game {
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

	pub fn get_or_create_game_loop(&mut self, map_name: &str, event_loop: &mut mio::EventLoop<Server>) -> Arc<RefCell<GameLoop>> {
        println!("{}", map_name);
		//This can handle all kinds of things. Checks last time user was inside, if too long it recreates. 
		//Checks the hashmap for the Gameloop. If not there, it creates a new one, adds it and returns it.
        let send = event_loop.channel();
        let _ = send.send(Msg::TextOutput(mio::Token(1), 2, "test".to_string()));
        let game_loop = self.game_loops.entry(map_name.to_string()).or_insert(Arc::new(RefCell::new(GameLoop::new(map_name, send))));
        game_loop.borrow_mut().start();
        game_loop.clone()
	}
}
