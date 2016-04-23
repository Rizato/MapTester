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
pub struct Item {
    pub tile: String,
    index: u32,
}

impl Item {
    pub fn new(tile: String, index: u32) -> Item {
        //Random tiles that need to be hardcoded
        let mut t = tile.replace("structures", "statics");
        t = t.replace("volcano", "volcano.1");
        t = t.replace("wiz/wyvern/hack/dungeon", "statics/well");
        Item {
            tile: t,
            index: index,
        }
    }
}

impl Controllable for Item {
    ///Called every game loop to update it
    fn update(&mut self, _: u8, _: u8, _: &Vec<bool>) -> Option<Vec<(mio::Token, u8, String)>> {
        None
    }
    ///Used when drawing the screen

    fn get_location(&self) -> u32 {
        self.index
    }

    ///Gets the artwork
    fn get_tile(&self) -> String {
        self.tile.clone()
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
        self.index == index
    }

    /// Whether or not to show this object. This obviously fails for very large objects that are
    ///off screen. Maybe I will handle that. 
    fn is_visible(&self, _: &GameMap) -> bool {
        true
    }

    ///This function subtracts from the hp of the player
    fn hurt(&mut self, _: i32) {
    }

    ///Pushes a command to the command queue for the mapuser
    fn push_command(&mut self, _: String) {
    }
    
    ///Puts an absolute X, Y as the movement goal for this mapuser
    fn set_movement(&mut self, _: u32) {
    }

    fn modify_connected_tiles(&mut self, _: u8, _: u8,  _: &Vec<bool>) {}

    fn get_type(&self) -> ControllableType {
        ControllableType::Item
    }
}
