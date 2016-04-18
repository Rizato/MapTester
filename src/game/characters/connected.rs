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
/// This module declares the item object. 

extern crate mio;

use game::characters::Controllable;
use game::characters::ControllableType;
use game::characters::Direction;
use game::gamemap::GameMap;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry::{Vacant, Occupied};


///Defines the RoadWall struct.
#[derive(Clone)]
pub struct RoadWall{
    pub tile: String,
    index: u32,
}

impl RoadWall {
    pub fn new(tile: String, tiles: &HashMap<String, i16>, index: u32) -> RoadWall {
        RoadWall{
            tile: RoadWall::find_corrected_tile(tile, tiles),
            index: index,
        }
    }

    ///This adjusts for the differences between map tile labels & the artwork file names
    fn find_corrected_tile(tile: String, tiles: &HashMap<String, i16>) -> String {
        let options = RoadWall::create_tile_options(&tile);
        for option in options {
            match tiles.get(&option) {
                None => {},
                Some(_) => {
                    return option.clone();
                },
            }
        }
        tile
    }

    ///This section is hideos. The path attributes from the 
    ///mapmaker do not match 1:1 with image paths. As a
    ///result, I have to translate between them. While I 
    ///would prefer this to be a lookup, there are just too
    ///many files for that. Sure, I could script it, if
    ///I had a way of getting all the map maker paths.
    ///Instead, I try to generate names where I can, and hardcode
    ///the known exceptions. Ugly AF.
    fn create_tile_options(tile: &str) -> Vec<String> {
        //Some hardcoded values when they are way off
        if tile == "roads/twisty_road" {
            println!("Had twisty_road");
            return vec!["roads/TwistyMntRoad".to_string();1];
        }
        
        let mut options = vec![];
        let mut t = tile.to_string();
        t = t.replace("earth_block", "earth_wall");
        options.push(t.clone());
        let (root, temp) = t.split_at(6);
        let mut r = root.to_string();
        if t.contains("walls/") {
            let mut parts: Vec<&str>= vec![];
            if temp.contains("brick_") {
                parts.push("brickwall");
            } else if temp.contains("_") {
                parts = temp.split("_").collect();
            } else if temp.contains("wall") {
                parts = temp.split("wall").collect();
            }
            r.push_str(parts[0]);
            r.push_str("/");
            options.push(format!("{}{}", r, temp));
        }
        if temp.contains("_") {
            let mut capital_case= String::new();
            let mut lower_case = String::new();
            for word in temp.split("_") {
                let (first, rest) = word.split_at(1);
                capital_case.push_str(&first.to_uppercase());
                capital_case.push_str(&rest);
                lower_case.push_str(&first);
                lower_case.push_str(&rest);
            }
            options.push(format!("{}{}", r, capital_case));
            options.push(format!("{}{}", r, lower_case));
        }
        //Pushing again just capitalizing the first letter
        let mut built = String::new();
        let (first, rest) = temp.split_at(1);
        built.push_str(&first.to_uppercase());
        built.push_str(&rest);
        options.push(format!("{}{}", r, built));
        options
    }
}

impl Controllable for RoadWall{
    ///Called every game loop to update it
    fn update(&mut self, width: u8, height: u8, blocked: &Vec<bool>) -> Option<Vec<(mio::Token, u8, String)>> {
        None
    }
    ///Used when drawing the screen

    fn get_location(&self) -> u32 {
        self.index
    }

    fn get_tile(&self) -> String {
        self.tile.clone()
    }

    ///Gets the correct direction tile for roads. This well connect any RoadWall objects
    ///of the same Road or Wall type.
    fn modify_connected_tiles(&mut self, width: u8, height: u8, roadwalls: &Vec<bool>) {
        if self.tile.to_lowercase().contains("bridge") 
            || self.tile.to_lowercase().contains("door") {
            return
        }
        let x = self.index % width as u32;
        let y = self.index / width as u32;
        let mut connected_map: u8 = 0;
        for dx in 0..3 {
            for dy in 0..3 {
                if dy == dx || (dx == 0 && dy == 2) || (dx == 2 && dy == 0)  {
                    continue;
                }
                let current_x = (x as i32) + (dx as i32) -1;
                let current_y = (y as i32) + (dy as i32) -1;
                if current_x >= 0 && current_y >= 0 {
                    if current_x as u32 >= width  as u32 || current_y as u32 >= height as u32 {
                        continue;
                    }
                    let i = (current_y as u32) * width as u32 + (current_x as u32);
                    if roadwalls[i as usize] {
                        if dx == 0 && dy == 1 {
                            //West
                            connected_map = connected_map | 1; 
                        } else if dx == 2 && dy == 1 {
                            //East
                            connected_map = connected_map | 2; 
                        } else if dx == 1 && dy == 0 {
                            //North
                            connected_map = connected_map | 8; 
                        } else if dx == 1 && dy == 2 {
                            //South
                            connected_map = connected_map | 4; 
                        }
                    }
                }
            }
        }
        let mut tile_append = String::new(); 
        if connected_map & 8 == 8 {
            tile_append.push('N');
        }
        if connected_map & 4 == 4 {
            tile_append.push('S');
        }
        if connected_map & 2 == 2 {
            tile_append.push('E');
        }
        if connected_map & 1 == 1 {
            tile_append.push('W');
        }
        self.tile.push_str(&tile_append);
    }

    ///This will tell if a tile is a road. True if road, false if wall. 
    fn get_type(&self) -> ControllableType {
        if self.tile.contains("roads") {
            ControllableType::Road
        } else {
            ControllableType::Wall
        }
    }

    ///Gets the Item size
    fn get_size(&self) -> (u32, u32) {
        (1,1)
    }

    ///Get the server token. As a non-player item, it has none.
    fn get_token(&self) -> Option<mio::Token> {
        None
    }
    ///Grabs the current hp value. Has none.
    fn get_hp(&self) -> Option<i32> {
        None
    }
    ///Changes the location on the map. 
    fn set_location(&mut self, index: u32) {
        self.index = index;
    }

    ///Checks to see if this value blocks the index in question. This does not take into account
    ///size of the artwork (Which I don't have access to on the server).
    fn does_block_index(&self, index: u32) -> bool {
        if self.index == index {
            match self.get_type() {
                ControllableType::Wall => {
                    true
                }, 
                _ => {
                    false
                },
            }
        } else {
            false
        }
    }

    /// Whether or not to show this object. This obviously fails for very large objects that are
    ///off screen. Maybe I will handle that. 
    fn is_visible(&self, center: u32, map: &GameMap) -> bool {
        true
    }

    ///This function subtracts from the hp of the player
    fn hurt(&mut self, damage: i32) {
    }

    ///Pushes a command to the command queue for the mapuser
    fn push_command(&mut self, command: String) {
    }
    
    ///Puts an absolute X, Y as the movement goal for this mapuser
    fn set_movement(&mut self, end: u32) {
    }
}
