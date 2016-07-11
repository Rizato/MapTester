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
use game::gamemap::GameMap;


///Defines the Item struct.
#[derive(Clone)]
pub struct Teleporter {
    pub map: String,
    index: u32,
    use_default: bool,
    x: u8, 
    y: u8, 
}

impl Teleporter {
    pub fn new(map: String, index: u32, default: bool, x: u8, y: u8) -> Teleporter {
        Teleporter{
            map: map,
            index: index,
            use_default: default,
            x: x, 
            y: y, 
        }
    }

    pub fn teleport(&self, token: mio::Token) -> (mio::Token, String, Option<(u8,u8)>){
        if self.use_default {
            (token.clone(), self.map.clone(), None)
        } else {
            (token.clone(), self.map.clone(), Some((self.x, self.y)))
        }
    }
}

impl Controllable for Teleporter{
    
    fn update(&mut self, _: u8, _: u8, _: &Vec<bool>) -> Option<Vec<(mio::Token, u8, String)>> {
        None
    }

    fn get_location(&self) -> u32 {
        self.index
    }

    fn get_tile(&self) -> String {
        "".to_string()
    }

    fn get_size(&self) -> (u32, u32) {
        (1,1)
    }

    fn get_token(&self) -> Option<mio::Token> {
        None
    }
    fn get_hp(&self) -> Option<i32> {
        None
    }
 
    fn set_location(&mut self, index: u32) {
        self.index = index;
    }

    fn does_block_index(&self, _: u32) -> bool {
        false
    }
 
    fn is_visible(&self, _: &GameMap) -> bool {
        false
    }

    fn hurt(&mut self, _: i32) {
    }

    fn push_command(&mut self, _: String) {
    }
    
    fn set_movement(&mut self, _: u32) {
    }

    fn modify_connected_tiles(&mut self, _: u8, _: u8,  _: &Vec<bool>) {}

    fn get_type(&self) -> ControllableType {
        ControllableType::Teleporter
    }

    fn get_viewport(&self) -> (u8, u8) {
        (0,0)
    }
}
