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
/// This module just declares the Moveable trait. 

extern crate mio;

pub mod player;
pub mod connected;
pub mod item;
pub mod teleporter;

use game::gamemap::GameMap;


/// Enum for the direction that a moveable object just went. Gets sent to the connection when
/// deciding what tile to draw.
#[derive(Clone)]
pub enum Direction {
    All,
    North,
    South,
    East,
    West,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

///This is used to indicate the type, since there is no instanceOf in rust
pub enum ControllableType{
    Road,
    Wall,
    Player,
    Item,
    Teleporter,
}

///This trait is used to define a set of functions for moveable objects. Helps with pathfinding.
pub trait Controllable {
    ///Called every game loop to update it
    fn update(&mut self, width: u8, height: u8, blocked: &Vec<bool>) -> Option<Vec<(mio::Token, u8, String)>>; 
    ///Used when drawing the screen
    fn get_location(&self) -> u32;
    ///Gets the artwork
    fn get_tile(&self) -> String;
    ///Gets the Item size
    fn get_size(&self) -> (u32, u32);
    ///Get the token
    fn get_token(&self) -> Option<mio::Token>;
    ///gets the current HP, if any
    fn get_hp(&self) -> Option<i32>;
    ///moves an object
    fn set_location(&mut self, index: u32);
    ///true if blocks player movement
    fn does_block_index(&self, index: u32) -> bool;
    ///whether or not to try to draw this on the MapScreen
    fn is_visible(&self, map: &GameMap) -> bool;
    ///Reduces a targets HP by damage (if supported)
    fn hurt(&mut self, damage: i32);
    ///Sets the end location for some movement
    fn set_movement(&mut self, end: u32); 
    ///Adds a command to the queue
    fn push_command(&mut self, command: String);
    ///Gets the correct direction tile for roads. This well connect any RoadWall objects
    ///of the same Road or Wall type.
    fn modify_connected_tiles(&mut self, width: u8, height: u8,  objects : &Vec<bool>);
    ///Returns a type because rust does not have instanceOf.
    fn get_type(&self) -> ControllableType;
}
