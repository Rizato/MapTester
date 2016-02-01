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

use game::characters::Moveable;
use game::gamemap::GameMap;

use std::collections::HashMap;
use std::collections::HashSet;

pub struct Player{
    id: i64,
    pub tile: String,
	hp: i32,
	max_hp: i32,
	pub name: String,
    pub speed: u8,
}

impl Player {
	pub fn new() -> Player {
		Player {
            id: 0,
            tile: "players/wizard.".to_string(),
			hp: 0,
			max_hp: 0,
			name: "empty".to_string(), 
            speed: 10,
		}
	}
	
	fn lowest_estimate(open: &HashSet<u32>, estimates: &mut HashMap<u32, u32>) -> u32{
		let mut min = 9999;
		let mut index_min = 0;
		for node in open.iter() {
			let mut val = estimates.entry(*node).or_insert(255); 
			if  *val < min {
				min = val.clone();
				index_min = node.clone();
			}
		}
		index_min
	}
	
	fn find_move(path: &HashMap<u32, u32>, end: u32) -> u32 {
		let mut current = end;
        loop {
			let temp = match path.get(&current) {
                Some(previous) => {
                    previous.clone()
                },
                None => {
                    break;
                }
            };
            if !path.contains_key(&temp) {
                break;
            }
            current = temp.clone();
            let x = temp % 30;
            let y = temp /30;
            println!("move {} {}", x, y);
		}
        let x = current % 30;
        let y = current /30;
        println!("actual {} {}", x, y);
		current
	}
}



impl Moveable for Player {
	//Computes the shortest path. Gives the next step in that. 
	fn path_next(map: &GameMap, start: u32, end: u32) -> Option<u32> {
        println!("Path!");
		
		//A* algorithm
		let mut closed = HashSet::new();
		//This should be a priority queue (or min-heap)
		let mut open = HashSet::new();
		//This is a map where each node records the node that came before it.
		//Why not use a doubly linked list?
		let mut path = HashMap::new();
		open.insert(start.clone());
		
		let mut score_from:HashMap<u32, u32> = HashMap::new();
		score_from.insert(start.clone(), 0);
		let mut estimate_to: HashMap<u32, u32> = HashMap::new();
		estimate_to.insert(start.clone(), Player::hueristic(map.width.clone(), start.clone(), end.clone()));
		while open.len() > 0 {
			//Grab start with the smallest estimate
			let current = Player::lowest_estimate(&open, &mut estimate_to);
			if current == end {
				//return the index of the first move 
                println!("Finished! {} {}", current, end);
				return Some(Player::find_move(&path, end.clone()));
			}
			open.remove(&current);
			closed.insert(current.clone());
			//Need to figure out how to get all neighbors
			let neighbors = Player::find_neighbors(current, map);
			for neighbor in neighbors.iter() {
                println!("Neighbor {}", neighbor);
				if closed.contains(neighbor) {
					continue;
				}
                println!("current {}", current);
                //This should always have a value...
				let possible_score = score_from.get(&current).unwrap() + 1 as u32;
				if !open.contains(neighbor) {
					path.insert(neighbor.clone(), current);
					open.insert(neighbor.clone());
                    score_from.insert(neighbor.clone(), possible_score.clone()); 
                    println!("possible score {}", possible_score);
					estimate_to.insert(neighbor.clone(),  possible_score + Player::hueristic(map.width.clone(), neighbor.clone(), end.clone()));	
				} else {
					match score_from.clone().get_mut(neighbor) {
						Some(ref mut value) => {
							if value.clone() > possible_score {
								continue;
							} else {
                                let mut n = path.entry(neighbor.clone()).or_insert(current.clone());
                                *n = current.clone();
								score_from.insert(neighbor.clone(), possible_score.clone());
								estimate_to.insert(neighbor.clone(),  possible_score + Player::hueristic(map.width.clone(), neighbor.clone(), end.clone()));	
							}
						},
						None => {
							path.insert(neighbor.clone(), current);
							score_from.insert(neighbor.clone(), possible_score.clone());
							estimate_to.insert(neighbor.clone(),possible_score + Player::hueristic(map.width.clone(), neighbor.clone(), end.clone()));
						},
					}
				}
			}
		}
		None
	}
	
	fn hueristic(width: u8, start: u32, end: u32) -> u32{
		//Just using pythagorean theorem to compute the shortest path.
		let dx = ((start % width as u32) as i32 - (end % width as u32) as i32).abs();
		let dy = ((start / width as u32) as i32 - (end / width as u32) as i32).abs();
		if dy == 0 {
			dx as u32
		} else if dx == 0 {
			dy as u32
		} else {
            println!("heuristic vals {} {}", dx, dy);
			((dx * dx + dy * dy) as f64).sqrt() as u32
		}
	}
	
	fn find_neighbors(index: u32, map: &GameMap) -> Vec<u32>{
		let width = map.width as u32;
		let x = index % width;
		let y = index / width;
		let mut neighbors = vec![];
		for dx in 0..3 {
			for dy in 0..3 {
                if dy == dx || (dx == 0 && dy == 2) || (dx == 2 && dy == 0)  {
                    continue;
                }
				let current_x = (x as i32) + (dx as i32) -1;
				let current_y = (y as i32) + (dy as i32) -1;
				if current_x >=0 && current_y >=0 {
                    let i = (current_y as u32) * width as u32 + (current_x as u32);
                    println!("neighbor {}", i);
					//if not blocked, add to neighbors
					let tiles = map.tiles.read().unwrap();
					if !tiles[i as usize].blocked {
						neighbors.push(i.clone());
					}
				}
			}
		}
        neighbors
	}
}
