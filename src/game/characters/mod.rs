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

pub mod player;

use game::gamemap::GameMap;


pub trait Moveable {
	///This will do the pathfinding, and give the next location for the player
	fn path_next(map: &GameMap, start: u32, end: u32) -> Option<u32>;
	fn hueristic(width: u8, start: u32, end: u32) -> u32;
	fn find_neighbors(index: u32, map: &GameMap) -> Vec<u32>;
}
