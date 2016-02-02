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
/// This module just declares the Moveable trait. 

pub mod player;

use game::gamemap::GameMap;

///This trait is used to define a set of functions for moveable objects. Helps with pathfinding.
pub trait Moveable {
	///This will do the pathfinding, and give the next location for the player
	fn path_next(map: &GameMap, start: u32, end: u32) -> Option<u32>;
    ///This gives an estimate for the total, for use in the hueristic
	fn hueristic(width: u8, start: u32, end: u32) -> u32;
    ///Returns a vector of indeices for valid neighbors
	fn find_neighbors(index: u32, map: &GameMap) -> Vec<u32>;
}
