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

///Defines the Projectile struct.
#[derive(Clone)]
pub struct Projectile{
    pub tile: String,
    pub speed: u8,
    pub target: mio::Token,
    index: u32,
    movement_ticks: u8,
    direction: Direction,
}

impl Projectile {
    ///Creates a new player. Defaults with the wizard tile, and a speed of 10 (1 movement every 20
    ///ms)
    pub fn new(token: mio::Token) -> Projectile {
        Projectile {
            tile: "spells/blizzard/snowball_meteor.".to_string(),
            speed: 10,
            index: 406,
            direction: Direction::South,
            target: token,
            movement_ticks: 0,
        }
    }
    
    ///This is an assisting function for moveable and the A* algorithm. It basically just tries to
    ///find the lowest value in the open tiles in the A* algorithm
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
    
    ///Given a map of tiles to the tile that led to it and the ending tile, it will go back through
    ///the map, finding the first move on the path
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
            //println!("move {} {}", x, y);
        }
        let x = current % 30;
        let y = current /30;
        //println!("actual {} {}", x, y);
        current
    }

    ///Computes the shortest path according to the A* algorithm. Gives the next step in the found path 
    pub fn path_next(map: &GameMap, start: u32, end: u32) -> Option<u32> {
        //println!("Path!");
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
        estimate_to.insert(start.clone(), Projectile::hueristic(map.width.clone(), start.clone(), end.clone()));
        while open.len() > 0 {
            //Grab start with the smallest estimate
            let current = Projectile::lowest_estimate(&open, &mut estimate_to);
            if current == end {
                //return the index of the first move 
                println!("Finished! {} {}", current, end);
                return Some(Projectile::find_move(&path, end.clone()));
            }
            open.remove(&current);
            closed.insert(current.clone());
            //Need to figure out how to get all neighbors
            let neighbors = Projectile::find_neighbors(current, map);
            for neighbor in neighbors.iter() {
                //println!("Neighbor {}", neighbor);
                if closed.contains(neighbor) {
                    continue;
                }
                //println!("current {}", current);
                //This should always have a value...
                let possible_score = score_from.get(&current).unwrap() + 1 as u32;
                if !open.contains(neighbor) {
                    path.insert(neighbor.clone(), current);
                    open.insert(neighbor.clone());
                    score_from.insert(neighbor.clone(), possible_score.clone()); 
                    //println!("possible score {}", possible_score);
                    estimate_to.insert(neighbor.clone(),  possible_score + Projectile::hueristic(map.width.clone(), neighbor.clone(), end.clone()));    
                } else {
                    match score_from.clone().get_mut(neighbor) {
                        Some(ref mut value) => {
                            if value.clone() > possible_score {
                                continue;
                            } else {
                                let mut n = path.entry(neighbor.clone()).or_insert(current.clone());
                                *n = current.clone();
                                score_from.insert(neighbor.clone(), possible_score.clone());
                                estimate_to.insert(neighbor.clone(),  possible_score + Projectile::hueristic(map.width.clone(), neighbor.clone(), end.clone()));    
                            }
                        },
                        None => {
                            path.insert(neighbor.clone(), current);
                            score_from.insert(neighbor.clone(), possible_score.clone());
                            estimate_to.insert(neighbor.clone(),possible_score + Projectile::hueristic(map.width.clone(), neighbor.clone(), end.clone()));
                        },
                    }
                }
            }
        }
        None
    }
    
    ///Gives a hueristic estimate by just doing the pythagorean theorem.
    fn hueristic(width: u8, start: u32, end: u32) -> u32{
        //Just using pythagorean theorem to compute the shortest path.
        let dx = ((start % width as u32) as i32 - (end % width as u32) as i32).abs();
        let dy = ((start / width as u32) as i32 - (end / width as u32) as i32).abs();
        if dy == 0 {
            dx as u32
        } else if dx == 0 {
            dy as u32
        } else {
            //println!("heuristic vals {} {}", dx, dy);
            ((dx * dx + dy * dy) as f64).sqrt() as u32
        }
    }
    
    ///Returns the found neighbors to a given index. Does one up, down, left and right.
    fn find_neighbors(index: u32, map: &GameMap) -> Vec<u32> {
        let width = map.width as u32;
        let x = index % width;
        let y = index / width;
        let mut neighbors = vec![];
        for dx in 0..3 {
            for dy in 0..3 {
                let current_x = (x as i32) + (dx as i32) -1;
                let current_y = (y as i32) + (dy as i32) -1;
                if current_x >=0 && current_y >=0 {
                    if current_x as u32 >= width || current_y as u32 >= map.height as u32 {
                        continue;
                    }
                    let i = (current_y as u32) * width as u32 + (current_x as u32);
                    //println!("neighbor {}", i);
                    neighbors.push(i.clone());
                }
            }
        }
        neighbors
    }
    ///Pushes a command to the command queue for the mapuser
    fn push_command(&mut self, command: String) {
    }
    
    ///Puts an absolute X, Y as the movement goal for this mapuser
    fn set_movement(&mut self, end: u32) {
    }

    ///Checks the end index against the movement goal index. If they are the same,
    /// it wipes out the movement goalt.
    fn clear_movement_if_at_destination(&mut self, end: u32) {
    }
    
    ///Grabs an available command from the queue. Manages movement by counting the number of cycles
    ///since last movment. This prefers movement over the top of the queue. SO basically
    /// 
    /// This checks the number of ticks, against a threshold. If it is greater or equal, and there is a
    ///movement goal set, it will always do movement. Otherwise, it will increment ticks, and grab
    ///the top command from the queue.
    pub fn get_command(&mut self, index: u32) -> Option<String> {
        self.index = index;
        if self.movement_ticks >= self.speed {
            self.movement_ticks = 0;
            Some(format!("ProjectileFindAndTrack {} {}", self.index, self.target.as_usize())) 
        } else {
            self.movement_ticks = if self.movement_ticks == 255 {255} else {self.movement_ticks+1};
            None
        }
    }
    
    pub fn get_tile(&self) -> String {
        let direction = match self.direction {
                                    Direction::South => {"S"},
                                    Direction::North => {"N"},
                                    Direction::East => {"E"},
                                    Direction::West => {"W"},
                                    Direction::NorthWest => {"NW"},
                                    Direction::NorthEast => {"NE"},
                                    Direction::SouthWest => {"SW"},
                                    Direction::SouthEast => {"SE"},
                                    _ => {"S"},
                                };
        format!("{}{}1",self.tile, direction)
        //format!("{}",self.tile)
    }

    fn does_move(&self) -> bool {
        true
    }

    pub fn set_direction(&mut self, dir: Direction) {
        self.direction = dir;
    }
}
