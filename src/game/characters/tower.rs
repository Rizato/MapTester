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
/// This module declares the player object. 
/// The player defines attributes such as the tile to represent the character,
/// the health, mana pool, character name and speed

extern crate mio;

use game::characters::Controllable;
use game::characters::Direction;
use game::gamemap::GameMap;

use std::collections::HashMap;
use std::collections::HashSet;

///Defines the Player struct.
#[derive(Clone)]
pub struct Tower{
    pub tile: String,
    hp: i32,
    max_hp: i32,
    launch_speed: u8,
    launch_ticks: u8,
    //This could be improved with a priority queue
    target_priorities: HashMap<mio::Token, u64>,
    last_priority: Option<mio::Token>
}

impl Tower {
    ///Creates a new player. Defaults with the wizard tile, and a speed of 10 (1 movement every 20
    ///ms)
    pub fn new() -> Tower {
        Tower {
            tile: "statics/soko_tower".to_string(),
            hp: 140,
            max_hp: 140, 
            launch_speed: 200,
            launch_ticks: 0,
            target_priorities: HashMap::new(),
            last_priority: None,
        }
    }
    ///Grabs an available command from the queue. Manages movement by counting the number of cycles
    ///since last movment. This prefers movement over the top of the queue. SO basically
    /// 
    /// This checks the number of ticks, against a threshold. If it is greater or equal, and there is a
    ///movement goal set, it will always do movement. Otherwise, it will increment ticks, and grab
    ///the top command from the queue.
    pub fn get_command(&mut self) -> Option<String> {
        let player = self.get_highest_priority();
        if self.launch_ticks >= self.launch_speed && player.is_some() {
            //Target the player with the highest priority
            self.launch_ticks = 0;
            Some(format!("TowerShoot {}", player.unwrap().as_usize()))
        } else {
            self.launch_ticks = if self.launch_ticks < 255 { self.launch_ticks +1} else {255};
            None
        }
    }
    
    fn get_highest_priority(&mut self) -> Option<mio::Token> {
        let mut max = 0;
        let mut conn: Option<mio::Token> = None;
        for (token, total) in &self.target_priorities {
            if total > &max {
                max = total.clone();
                conn = Some(token.clone())
            }
        }
        conn
    }
    
    pub fn push_tokens(&mut self, tokens: Vec<mio::Token>) {
        let mut temp = HashMap::new();
        for conn in self.target_priorities.keys() {
            if tokens.contains(&conn) {
                let total = self.target_priorities.get(&conn).unwrap().clone();
                println!("Total {} conn {}" , total, conn.as_usize());
                temp.insert(conn.clone(), total +1);
            } 
        }
        self.target_priorities = temp.clone();

        for conn in tokens {
            if !self.target_priorities.contains_key(&conn) {
                self.target_priorities.insert(conn, 1);
            }
        }
    }
    
    pub fn get_tile(&self) -> String {
        "statics/soko_tower".to_string()
    }

    fn does_move(&self) -> bool {
        false
    }

    fn set_direction(&mut self, dir: Direction) {
        return
    }
}
