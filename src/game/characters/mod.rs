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

pub mod player;
pub mod tower;
pub mod projectile;

use game::gamemap::GameMap;

/// Enum for the direction that a moveable object just went. Gets sent to the connection when
/// deciding what tile to draw.
#[derive(Clone)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

///This trait is used to define a set of functions for moveable objects. Helps with pathfinding.
pub trait Controllable {
    ///This will do the pathfinding, and give the next location for the player
    fn path_next(map: &GameMap, start: u32, end: u32) -> Option<u32>;
    ///This gives an estimate for the total, for use in the hueristic
    fn hueristic(width: u8, start: u32, end: u32) -> u32;
    ///Returns a vector of indeices for valid neighbors
    fn find_neighbors(index: u32, map: &GameMap) -> Vec<u32>;
    ///Grabs the command
    fn get_command(&mut self) -> Option<String>;
    ///Removes the movement value if there is one
    fn clear_movement_if_at_destination(&mut self, end: u32);
    ///Sets a movement position for an object
    fn set_movement(&mut self, end: u32);
    ///Adds a command to the queue
    fn push_command(&mut self, command: String);
    ///Returns the tile artwork for the character
    fn get_tile(&self) -> String;
    ///Returns true if this is a moving object
    fn does_move(&self) -> bool;
    ///Sets the controllable's direction
    fn set_direction(&mut self, dir: Direction);
}
