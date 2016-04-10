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

extern crate mio;
extern crate slab;
extern crate xml;

use game::characters::Controllable;
use game::characters::Direction;
use game::characters::player::Player;
use game::characters::tower::Tower;
use game::characters::projectile::Projectile;

use std::sync::RwLock;
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};

use self::slab::Index;

/// This module holds all the map related stuff. It has the GameMap itself, along with the
/// MapScreen, ScreenObjects, 

///This is the map. It holds all of the terratin, and all of the objects and such. 
///It also holds the x,y of the start value. This is only temporary until I get objects for start
///values
#[derive(Clone)]
pub struct GameMap {
    pub width: u8,
    pub height: u8,
    pub tiles: Arc<RwLock<Vec<MapTile>>>,
    start_x: u8,
    start_y: u8, 
}

impl GameMap {
    /// This currently creates a generic map. It will eventually load a map by filename and turn
    /// that into a valid MapObject
    pub fn new(mapname: &str) -> Result<GameMap, String> {
        if !GameMap::maps_exist(mapname) {
            return Err("Map Not Found".to_string());
        }
        match GameMap::parse_tiles(mapname) {
            Ok(mut map) => {
                //Still throwing the tile in because right now it is setup for that. 
                //let t_index = 405;
                //map.tiles[t_index as usize].user = Some(MapUser::new(None,Commandable::T(Tower::new())));
                //map.tiles[t_index as usize].blocked = true;
                //let mut ti = Arc::new(RwLock::new(tiles));
                Ok(map)
            },
            Err(s) => {
                Err(s)
            },
        }
    }
    
    fn maps_exist(path: &str) -> bool {
        match File::open(path) {
            Err(_) => {
               false 
            }, 
            Ok(_) => {
                true
            }
        }
    }

    fn parse_tiles(path: &str) -> Result<GameMap, String>{
        println!("Parsing! {}", path);
        match File::open(path) {
            Err(_) => {
               Err("Failed to parse".to_string()) 
            },
            Ok(file) => {
                let buf = BufReader::new(file);
                let parser = EventReader::new(buf);
                let mut header = false;
                let mut tiles: Vec<MapTile> = vec![];
                let mut width: u8 = 0;
                let mut height: u8 = 0;
                let mut start_x: u8 = 0;
                let mut start_y: u8 = 0;
                let mut out_of_bounds_tile = String::new();
                for event in parser {
                    match event { 
                        Ok(XmlEvent::StartElement {name, attributes, ..}) => {
                            if name.local_name == "header" {
                                println!("Header");
                                header = true;
                                for attr in attributes {
                                    if attr.name.local_name == "width" {
                                        match attr.value.parse::<u8>() {
                                            Ok(w) => {
                                                width = w;
                                                println!("Width {}", width);
                                            },
                                            Err(_) => {
                                                return Err("Bad Width".to_string());
                                            },
                                        }
                                    } else if attr.name.local_name == "height" {
                                        match attr.value.parse::<u8>() {
                                            Ok(h) => {
                                                height = h;
                                                println!("Height {}", height);
                                            },
                                            Err(_) => {
                                                return Err("Bad Height".to_string());
                                            },
                                        }
                                    }
                                }
                                //Define header behavior
                                if height > 0 && width > 0 {

                                } else {
                                    return Err("Invalid dimensions".to_string());
                                }
                            } else if name.local_name == "arch" {
                                println!("Arch");
                                if header {
                                    let mut terrain = String::new();
                                    let mut is_terrain = false;
                                    let mut is_oob = false;
                                    for attr in attributes {
                                        //TODO Handle out of bounds terrain.
                                        println!("head-arch {}", attr.name.local_name);
                                        if attr.name.local_name == "name" &&
                                            attr.value == "terrain" {
                                                is_terrain = true;
                                        } else if attr.name.local_name == "path" { 
                                            terrain = attr.value.clone();
                                        } else if attr.name.local_name == "name" &&
                                         attr.value == "oob-terrain" {
                                            is_oob = true;
                                        }
                                    }
                                    if is_terrain {
                                        let size = width as u32 * height as u32;
                                        for x in 0..size {
                                            let mut tile = MapTile::new(terrain.clone());
                                            tiles.push(tile);
                                        }
                                    } else if is_oob {
                                        out_of_bounds_tile = terrain;
                                    }
                                } else {
                                    //TODO
                                    //Read the loc. 
                                    //If it is a rectangle, apply that tile to the
                                    //  entire area as the terrain.
                                    //Else add it to the array of map items  (need to refactor maps)
                                    let mut terrain: String = "".to_string();
                                    let mut rect_x: u8 = 0;
                                    let mut rect_y: u8 = 0;
                                    let mut rect_w: u8 = 0;
                                    let mut rect_h: u8 = 0;
                                    for attr in attributes {
                                        if attr.name.local_name == "path" {
                                            terrain = attr.value;
                                        } else if attr.name.local_name =="loc" {
                                            let split: Vec<&str>= attr.value.split(" ").collect();
                                            if split.len() == 4 {
                                                rect_x = split[0].parse::<u8>().unwrap();
                                                rect_y = split[1].parse::<u8>().unwrap();
                                                rect_w = split[2].parse::<u8>().unwrap();
                                                rect_h = split[3].parse::<u8>().unwrap();
                                            } else if split.len() == 2 {
                                                rect_x = split[0].parse::<u8>().unwrap();
                                                rect_y = split[1].parse::<u8>().unwrap();
                                                rect_w = 1;
                                                rect_h = 1;
                                            }
                                        }
                                    }
                                    for x in rect_x..(rect_x+rect_w) {
                                        for y in rect_y..(rect_y+rect_h) {
                                            let index: usize = (y as usize * width as usize) as usize + x as usize;
                                            tiles[index].tile = terrain.clone();
                                            tiles[index].blocked = false;
                                        }
                                    }
                                }
                            } else if name.local_name == "int" {
                                let mut name = String::new(); 
                                let mut val = 0;
                                for attr in attributes {
                                    if attr.name.local_name == "name" {
                                        name = attr.value;
                                    } else if attr.name.local_name == "value" {
                                        val = attr.value.parse::<u8>().unwrap();
                                    }
                                }
                                if name.contains("startX") {
                                    start_x = val;
                                } else if name.contains("startY") {
                                    start_y = val;
                                }
                            }
                        },
                        Ok(XmlEvent::EndElement {name}) => {
                            if name.local_name == "header" {
                                println!("End header");
                                header = false;
                                for index in 0..tiles.len() {
                                    println!("{} {}", tiles[index].tile, out_of_bounds_tile);
                                    if tiles[index].tile == out_of_bounds_tile {
                                        tiles[index].blocked = true;
                                    }
                                }
                            }
                        },
                        _ => {

                        },
                    }
                }
                Ok(GameMap{
                    width: width,
                    height: height,
                    tiles: Arc::new(RwLock::new(tiles)),
                    start_x : start_x,
                    start_y : start_y,
                })
            },
        }
    }

    ///Grabs the MapTile at the given index
    fn get_user(&self, index: u32) -> MapTile {
        println!("tile: {}", index);
        let mut tiles = self.tiles.read().unwrap();
        tiles[index as usize].clone()
    }

    ///Moves a user from one tile to another, replaces the direction witht he given direction
    fn move_user(&mut self, o:u32, n:u32, d: Direction) -> bool {
        println!("Move user");
        //println!("{}", n);
        let old = self.get_user(o); 
        let mut tiles = self.tiles.write().unwrap();
        let ref mut new = tiles[n as usize];
        if !new.blocked {
            match new.user {
                Some(_) => {
                    false
                },
                None => {
                    println!("None");
                    let mut u = old.user.clone().unwrap();
                    u.clear_movement_if_at_destination(n);
                    u.set_direction(d.clone());
                    new.user = Some(u);
                    new.blocked = true;
                    true
                }
            }
        } else {
            false
        }
    }

    ///Sets a new tile for the user
    fn change_skin(&mut self, index: u32, skin: &str ) {
        println!("Changing skin to {}", skin);
        let mut tiles = self.tiles.write().unwrap();
        if tiles[index as usize].blocked {
            match tiles[index as usize].user {
                Some(ref mut user) => {
                    match user.player {
                        Commandable::P(ref mut player) => {
                            player.tile = format!("players/{}.", skin.to_string());
                        },
                        _ => {},
                    }
                },
                None =>{},
            }
        }
    }

    /// Removes the tile at the given index
    fn wipe_user(&mut self, o: u32) {
        println!("Wipe User");
        //TODO make sure the tile was not empty before (Cause if it was empty and was blocked it is
        //blocked by an object, and we don't want to unblock.
        let mut tiles = self.tiles.write().unwrap();
        let ref mut old = tiles[o as usize];
        old.user = None;
        old.blocked = false;
    }
    
    ///This just does the given damage to the users's health 
    fn hurt_user(&mut self, token: mio::Token, damage: i32) {
        let (x, y) = self.find_tile_with_token(token).unwrap();
        let index = y as u32 * self.width as u32 + x as u32;
        let mut tiles = self.tiles.write().unwrap();
        match tiles[index as usize].user {
            Some(ref mut user) => {
                match user.player {
                    Commandable::P(ref mut player) => {
                        player.hurt(damage);
                    },
                    _ => {},
                } 
            },
            _ => {},
        };
    
        
    }
    
    ///Moves a user from one tile to another, replaces the direction witht he given direction
    fn move_projectile(&mut self, o:u32, n:u32, d: Direction) -> bool {
        println!("Move user");
        //println!("{}", n);
        let old = self.get_user(o); 
        let mut tiles = self.tiles.write().unwrap();
        let ref mut new = tiles[n as usize];
        match new.projectile {
            Some(_) => {
                false
            },
            None => {
                println!("None");
                let mut u = old.projectile.clone().unwrap();
                u.set_direction(d.clone());
                new.projectile = Some(u);
                true
            }
        }
    }

    /// Removes the tile at the given index
    fn wipe_projectile(&mut self, o: u32) {
        println!("Wipe User");
        //TODO make sure the tile was not empty before (Cause if it was empty and was blocked it is
        //blocked by an object, and we don't want to unblock.
        let mut tiles = self.tiles.write().unwrap();
        let ref mut old = tiles[o as usize];
        old.projectile = None;
    }

    /// Adds the command from the client to the user object
    pub fn push_command(&mut self, token: mio::Token, command: String) {
        println!("push command");
        let (x, y) = self.find_tile_with_token(token.clone()).unwrap();
        let start = y as u32 * self.width as u32+ x as u32;
        let mut tiles = self.tiles.write().unwrap();
        println!("Command {}", command);
        if command.starts_with("mouse") {
            let parts: Vec<&str> = command.split_whitespace().collect();
            //Mouse click x,y
            let mx = parts[1].parse::<i32>().unwrap();
            let my = parts[2].parse::<i32>().unwrap();
            //old x,y
            let oy = (&start) / (self.width as u32);
            let ox = (&start) % (self.width as u32);
            //change in x,y. -6 cause user is always in middle of screen, no matter the click.
            let dx = if ox as i32 + mx > 6 { ox + mx as u32 -6 } else {0};
            let dy = if oy as i32 + my > 6 { oy + my as u32 -6 } else {0};
            println!("Move to x{} y{}", dx, dy);
            let end = dy * self.width as u32 + dx;
            //tiles[start as usize].user.unwrap().set_movement(end.clone());
            match tiles[start as usize].user {
                Some(ref mut u) => {
                    println!("Set movement");
                    u.set_movement(end.clone());
                }, 
                None => {

                }
            };
        } else {
            match tiles[start as usize].user {
                Some(ref mut u) => {
                    u.push_command(command);
                }, 
                None => {

                }
            };
        }
    }

    /// Returns the x,y value of a token
    fn find_tile_with_token(&self, token: mio::Token) -> Option<(u32, u32)> {
        let tiles = self.tiles.read().unwrap();
        let len = tiles.len();
        for t in 0..len {
            match tiles[t as usize].user {
                Some(ref u) => {
                    if u.token == Some(token) {
                       let y = (t as u32) / self.width as u32;
                       let x = (t as u32) % self.width as u32;
                       return Some((x, y));
                    }
                },
                None => {},
            }
        }
        None
    }
    
    /// This goes through all connections, tries to read off the queue, and then executes each
    ///command, possibly returning a tailored response 
    pub fn execute(&mut self, conns: &[mio::Token]) -> Vec<(mio::Token, u8, String)> {
        //Go through all users. 
        //Go through all monsters & towers
        //Go through all spells and projectiles
        //Resolve any combat/damage
        //Add responses for action specific to players involved
        //return the vec
        //let x  = self.tower_index % self.width as u32;
        //let y  = self.tower_index / self.width as u32;
        let width = self.width;
        let height = self.height;
        
        //Pushing all tokens to the tower   
        //{
        //    let mut players= vec![];
        //    //Grabbing all of the players around the tower index
        //    let mut tiles = self.tiles.write().unwrap();
        //    let start_x = if x > 8 { x-8} else {0};
        //    let end_x = if x +10 < self.width as u32 {x + 10} else {self.width as u32};
        //    let start_y = if y > 8 { y-8} else {0};
        //    let end_y = if y +10 < self.height  as u32 {y + 10} else {self.height as u32};
        //    for i in start_x..end_x {
        //        for j in start_y..end_y {
        //            let ref player = tiles[j  as usize * self.width as usize + i as usize];
        //            match player.user {
        //               Some(ref u) => {
        //                   match u.token {
        //                       Some(ref t) => {
        //                           &players.push(t.clone());
        //                       },
        //                       _ => {},
        //                   }
        //                }, 
        //                _ => {},
        //            }
        //        }
        //    }
        //    match tiles[self.tower_index as usize].user {
        //        Some(ref mut user) => {
        //            match user.player {
        //                Commandable::T(ref mut tower) => {
        //                    tower.push_tokens(players); 
        //                },
        //                _ => {},
        //            } 
        //        },
        //        _ => {},
        //    };
        //}
        let mut retval = vec![];
        //Looping through all tiles
        let size = (height as usize * width as usize) as usize;
        for i in 0..size {
            let mut t = None;
            let mut command = None;
            //let mut projectile_command = None;
            {
        //        //Gets the command the mapuser on this tile (if one exists)
                let mut tiles = self.tiles.write().unwrap();
                command = match tiles[i].user {
                    Some(ref mut user) => {
                        t = user.token.clone();
                        match user.player {
                            Commandable::P(ref mut player) => {
                                player.get_command()
                            },
                            Commandable::T(ref mut tower) => {
                                tower.get_command()
                            },
                        }
                    },
                    _ => { 
                        None
                    },
                };
        //        //gets the command for a projectile on this tile, if one exists
        //        projectile_command = match tiles[i].projectile {
        //            Some(ref mut p) => {
        //                p.get_command(i as u32)
        //            },
        //            _ => {None},
        //        };
            }
            //Executing command from player or tower
            match command {
                Some(c) => {
                    println!("Executing command");
                    match self.execute_command(t, c) {
                        Some(responses) => {
                            for x in 0..responses.len() {
                                let (token, style, response) = responses[x].clone();
                                retval.push((token, style, response));
                            }
                        },
                        None => {},
                    };
                },
                None => {},
            }
        //    //executing command from projectile
        //    match projectile_command {
        //        Some(c) => {
        //            println!("command {}", c);
        //            match self.execute_command(None, c) {
        //                Some(responses) => {
        //                    for x in 0..responses.len() {
        //                        let (token, style, response) = responses[x].clone();
        //                        retval.push((token, style, response));
        //                    }
        //                },
        //                None => {},
        //            };
        //        },
        //        None => {},
        //    }
        }
        retval
    }
    
    ///Executes a given command. Generates a possibly generates a vector of responses.
    fn execute_command(&mut self, token: Option<mio::Token>, command: String ) -> Option<Vec<(mio::Token, u8, String)>> {
        //println!("Execute Command");
        match token {
            Some(t) => {
                let (x, y) = self.find_tile_with_token(t.clone()).unwrap();
                let index = y as u32 * self.width as u32 + x as u32;
                if command.starts_with("end") {
                    let parts: Vec<&str> = command.split_whitespace().collect();
                    let end = parts[1].parse::<u32>().unwrap();
                    println!("Execute path: {} {}", index, end);
                    let e = Player::path_next(&self, index.clone(), end);
                    match e {
                        Some(user_end) => {
                            let dx = user_end % self.width as u32;
                            let dy = user_end / self.width as u32;
                            // Since the primary objective is east/west I will lean towards e/w when moving diagonally
                            let mut dir = Direction::South;
                            if dx > x as u32  {
                                dir = Direction::East;
                            } else if dx < x as u32 {
                                dir = Direction::West;
                            } else if dy < y as u32 {
                                dir = Direction::North;
                            } 
                            if self.move_user(index.clone(), user_end, dir) {
                                self.wipe_user(index);
                            }
                            None
                        },
                        None => {
                            Some(vec![(t.clone(), 5,  "No Path Found".to_string()); 1])
                        },
                    }
                } else if command.starts_with("#skin "){
                    let parts: Vec<&str> = command.split(" ").collect();
                    if parts.len() > 1 {
                       self.change_skin(index.clone(), parts[1]); 
                       Some(vec![(t.clone(), 3,  "Skin Changed".to_string()); 1])
                    } else {
                        None
                    }
                }else {
                    //System message
                    println!("{}", command);
                    Some(vec![(t.clone(), 5,  "Bad command".to_string()); 1])
                }
            },
            None => {
                //Command from tower. Picks a target to shoot. Creates a new projectile
                if command.starts_with("TowerShoot") {
                    let parts: Vec<&str> = command.split_whitespace().collect();
                    let t = parts[1].parse::<usize>().unwrap();
                    let conn = mio::Token::from_usize(t);
                    let mut tiles = self.tiles.write().unwrap();
                    tiles[406].projectile = Some(Projectile::new(conn.clone()));
                    Some(vec![(conn,3 ,"Tower has fired on you!".to_string())])
                    //Moves the projectile located at the given index, towards the user with the given token
                } else if command.starts_with("ProjectileFindAndTrack") {
                    println!("Projectile Should be moving");
                    //This whole projectile thing is horrible. Absolutely horrible.
                    //I am just trying to get it demo-able though. So quick here we come.
                    let parts: Vec<&str> = command.split_whitespace().collect();
                    let start = parts[1].parse::<u32>().unwrap();
                    let x = start % self.width as u32;
                    let y = start / self.width as u32;
                    let t = parts[2].parse::<usize>().unwrap();
                    let conn = mio::Token::from_usize(t);
                    let (end_x, end_y) = self.find_tile_with_token(conn.clone()).unwrap();
                    let index = end_y as u32 * self.width as u32 + end_x as u32;
                    println!("Start: {}, end: {}", start, index);
                    let next = Projectile::path_next(self, start.clone(), index);
                    match next {
                        Some(i) => {
                        println!("Next: {}", i);
                            if index == i {
                                self.hurt_user(conn.clone(), 50);
                                self.wipe_projectile(start);
                                Some(vec![(conn, 3, "Projectile smashed into you".to_string()); 1])
                            } else {
                                let dx = i % self.width as u32;
                                let dy = i / self.width as u32;
                                // Since the primary objective is east/west I will lean towards e/w when moving diagonally
                                let mut dir = Direction::South;
                                if dx > x && dy == y   {
                                    dir = Direction::East;
                                } else if dx < x && dy == y {
                                    dir = Direction::West;
                                } else if dx > x && dy > y {
                                    dir = Direction::SouthEast;
                                } else if dx > x && dy < y {
                                    dir = Direction::NorthEast;
                                }  else if dx < x && dy > y {
                                    dir = Direction::SouthWest;
                                } else if dx < x && dy < y {
                                    dir = Direction::NorthWest;
                                } else if dy < y && dx == x {
                                    dir = Direction::North;
                                } 
                                if self.move_projectile(start.clone(), i, dir) {
                                    self.wipe_projectile(start);
                                }
                                None
                            }
                        },
                        _ => {None},
                    }
                } else {
                    None
                }
            }
        }
    }
    
    /// This pulls the HP from health. 
    pub fn get_hp(&self, token: mio::Token) -> Option<i32> {
        match self.find_tile_with_token(token.clone()) {
            Some((x,y)) => {
                let tiles = self.tiles.read().unwrap();
                let ref tile = tiles[y  as usize * self.width as usize + x as usize];
                match tile.user {
                    Some(ref user) => {
                        match user.player {
                            Commandable::P(ref player) => {
                                Some(player.hp as i32)
                            },
                            _ => {
                                None
                            },
                        }
                    },
                    _ => {
                    None
                    },
                }
            },
            None=> { 
                None
            },
        }
    }
    
    /// This generates a new MapScreen based on the location of the given connection's user
    pub fn send_portion(&self, token: mio::Token) -> Option<MapScreen> {
        //println!("Send Portion");
        //This sends the squares around the user, which will always be centered in the screen.
        match self.find_tile_with_token(token.clone()) {
            Some((x, y)) => {
                Some(MapScreen::new(self, x, y))
            },
            None => {
                None
            },
        }
    }

    /// Adds a player to the map. Puts it at the starting location.
    pub fn add_player(&mut self, token: mio::Token, name:String) {
        println!("Add Player");
        let sx = self.start_x.clone();
        let sy = self.start_y.clone();
        self.add_player_at(MapUser::new(Some(token.clone()), Commandable::P(Player::new(name))),
        sx, sy, Direction::All);
    }

    ///Recursively searches for an open location. Sadly this is a horrible algorithm, and will build characters
    ///all in one direction before it tries any of the others. Ugly. What really kills it is that
    ///it exectures on the game loop & stops everything while it searches. If the x direction is
    ///full, it will take forever.
    fn add_player_at(&mut self, user: MapUser, x: u8, y: u8, direction: Direction) -> bool {
        println!("adding at {} {}", x, y);
        let mut start_user = None;
        let mut blocked = false;
        if x >= 0 && x < self.width && y >=0 && y < self.height {
            {
                let mut tiles = self.tiles.write().unwrap();
                let ref mut start = tiles[y  as usize * self.width as usize + x as usize];
                start_user = start.user.clone();
                blocked = start.blocked;
            }
            if !blocked {
                match start_user {
                   None => {
                        println!("Open tile at {} {}", x, y);
                        let mut tiles = self.tiles.write().unwrap();
                        let ref mut start = tiles[y  as usize * self.width as usize + x as usize];
                        start.user = Some(user);
                        start.blocked = true;
                        true
                   },
                   Some(_) => {
                       match direction {
                            Direction::All => { 
                                (x > 0 && self.add_player_at(user.clone(), x-1, y, Direction::West))
                                    ||  self.add_player_at(user.clone(), x+1, y, Direction::East) 
                                    || (y > 0 && self.add_player_at(user.clone(), x, y-1, Direction::North))
                                    || self.add_player_at(user.clone(), x, y+1, Direction::South)
                            },
                            Direction::East=> { 
                                 self.add_player_at(user.clone(), x + 1, y, Direction::East)
                            },
                            Direction::West=> { 
                                 x > 0 && self.add_player_at(user.clone(), x - 1, y, Direction::West)
                                
                            },
                            Direction::South=> { 
                                 self.add_player_at(user.clone(), x, y+1, Direction::South)
                            },
                            Direction::North=> { 
                                y > 0 && self.add_player_at(user.clone(), x, y-1, Direction::North)
                            },
                            _ => {
                                false
                            },
                       }
                   },
                }
            } else {
                match direction {
                     Direction::All => { 
                             (x > 0 && self.add_player_at(user.clone(), x-1, y, Direction::West)) 
                             ||  self.add_player_at(user.clone(), x+1, y, Direction::East) 
                             || (y > 0 && self.add_player_at(user.clone(), x, y-1, Direction::North))
                             || self.add_player_at(user.clone(), x, y+1, Direction::South)
                     },
                     Direction::East=> { 
                          self.add_player_at(user.clone(), x + 1, y, Direction::East)
                     },
                     Direction::West=> { 
                          x > 0 && self.add_player_at(user.clone(), x - 1, y, Direction::West)
                     },
                     Direction::South=> { 
                          self.add_player_at(user.clone(), x, y+1, Direction::South)
                     },
                     Direction::North=> { 
                          y > 0 && self.add_player_at(user.clone(), x, y-1, Direction::North)
                     },
                     _ => {
                         false
                     },
                }
            }
        } else {
            false
        }
        
    }

    /// Removes a player from the map. 
    pub fn remove_player(&mut self, token: mio::Token) {
        println!("Remove Player");
        let (x, y) = self.find_tile_with_token(token.clone()).unwrap();
        let mut tiles = self.tiles.write().unwrap();
        let index = y as usize * self.width as usize + x as usize; 
        let ref mut t = tiles[index];
        t.user = None;
        t.blocked = false;
    }
}

/// A single tile option, which optionally holds a user. Holds an image tile, and whther the tile
/// is blocked or not.
#[derive(Clone)]
pub struct MapTile{
    //No position, because position is determined by the position in vector
    pub tile: String,
    pub user: Option<MapUser>,
    pub blocked: bool,
    pub projectile: Option<Projectile>,
    //TODO add a Vec<MapItem>
}

impl MapTile {
    fn new(tile: String) -> MapTile {
        MapTile{
            tile: tile,
            user: None,
            blocked: false,
            projectile: None,
        }
    }
}

#[derive(Clone)]
pub enum Commandable {
    P(Player),
    T(Tower),
}

///Is a controllable thing on the map. Has a tile, which does not hold a direction(to be added
///later). It also holds a queue of directions, and the player object at the location. Lastly, it
///holds some movement helper values. That is, the final destination of the user initiated
///movement, as well as the number of cycles since the last move. This is used in conjunction witht
///he player Speed value.
#[derive(Clone)]
pub struct MapUser{
    player: Commandable,
    token: Option<mio::Token>,
}

impl  MapUser {
    ///This creates a new map user object, with some defaults. Takes in a player object and the
    ///token for the connection
    fn new(token: Option<mio::Token>, player: Commandable) -> MapUser {
       MapUser {
            token: token, 
            player: player,
       }
    }

    fn does_move(&self) -> bool {
        match self.player {
            Commandable::P(ref player) => {
                player.does_move()
            },
            _ => {
                false
            },
        }
    }
    
    fn get_tile(&self) -> String {
        match self.player {
            Commandable::P(ref player) => {
                player.get_tile()
            },
            Commandable::T(ref tower) => {
                tower.get_tile()
            },
        }
    }
    
    fn path_next(&self, map: &GameMap, start: u32, end: u32) -> Option<u32> {
        match self.player {
            Commandable::P(_) => {
                Player::path_next(map, start, end)
            },
            _ => {
                None
            }
        }
    }
    ///This gives an estimate for the total, for use in the hueristic
    fn hueristic(&self, width: u8, start: u32, end: u32) -> u32 {
        match self.player {
            Commandable::P(ref player) => {
                Player::hueristic(width, start, end)
            },
            _ => {
                0
            }
        }
    }
    ///Returns a vector of indeices for valid neighbors
    fn find_neighbors(&self, index: u32, map: &GameMap) -> Vec<u32> {
        match self.player {
            Commandable::P(ref player) => {
                Player::find_neighbors(index, map)
            },
            _ => {
                vec![]
            }
        }
    }
    ///Grabs the command
    fn get_command(&mut self, index: u32) -> Option<String> {
        match self.player {
            Commandable::P(ref mut player) => {
                player.get_command()
            },
            Commandable::T(ref mut tower) => {
                tower.get_command()
            }
        }
    }
    ///Removes the movement value if there is one
    fn clear_movement_if_at_destination(&mut self, end: u32) {
        match self.player {
            Commandable::P(ref mut player) => {
                player.clear_movement_if_at_destination(end)
            },
            _ => {},
        }
    }
    ///Sets a movement position for an object
    fn set_movement(&mut self, end: u32) {
        match self.player {
            Commandable::P(ref mut player) => {
                player.set_movement(end)
            },
            _ => {},
        }
    }
    ///Adds a command to the queue
    fn push_command(&mut self, command: String) {
        match self.player {
            Commandable::P(ref mut player) => {
                player.push_command(command)
            },
            _ => {},
        }
    }

    fn set_direction(&mut self, direction: Direction) {
        match self.player {
            Commandable::P(ref mut player) => {
                player.set_direction(direction)
            },
            _ => {},
        }
    }
}

///This is just a screen object. It just holds and x,y and a tile.
#[derive(Clone)]
pub struct ScreenObject {
    pub tile: String,
    pub x: u8,
    pub y: u8,
}

impl ScreenObject {
    ///Creates a new screen object with the given tile, x and y
    fn new(tile: String, x: u8, y:u8) -> ScreenObject{
        ScreenObject{
            tile: tile,
            x: x,
            y: y,
        }
    }
}

///This struct just holds a tile
#[derive(Clone)]
pub struct ScreenTerrain {
    pub tile: String,
}

impl ScreenTerrain {
    ///Creates a new tile struct
    fn new(tile: String) -> ScreenTerrain {
        ScreenTerrain {
            tile: tile,
        }
    }
}

///Holds a vector of terrain objects, and a series of objects. The terrain is just a vector,
///representing a 15x15 matrix. The screen objects on the other hand denote their x and y
///specifically, rather than by their position in the array.
#[derive(Clone)]
pub struct MapScreen {
    //15x15 vector. 
    pub terrain: Vec<ScreenTerrain>,
    //User art at 7,7
    pub objects: Vec<ScreenObject>,
}

impl MapScreen {
    ///generates a new MapScreen based on the map and a given x & y. This will grab the 15x15
    ///matrix centered on the given x and y. Any spaces beyond the boundaries of the map is replaced
    ///with "empty" tiles
    pub fn new(map: &GameMap, x: u32, y: u32) -> MapScreen {
        let startx: isize = x as isize -7;
        let starty: isize = y as isize -7;
        let mut ter = Vec::with_capacity(225);
        let mut obj = vec![];
        //If coords are valid we will actually draw something
        let empty = ScreenTerrain::new("terrain/empty".to_string());
        if map.width as u32 > x && map.height as u32 > y {
            for i in 0..15 {
                for j in 0..15 {
                    if startx+i >= 0 && startx+(i as isize) < (map.width as isize) && starty+(j as isize) >= 0 && starty+(j as isize) < (map.height as isize) {
                        //get the tile from the map
                        let index= ((starty +j) * (map.width as isize)+ (startx+i)) as usize;
                        let tiles = map.tiles.read().unwrap();
                        //clone the map tile
                        let tile = tiles[index as usize].clone();
                        //Add the terrain from the tile
                        ter.push(ScreenTerrain::new(tile.tile.clone()));
                        match tile.user {
                            Some(u) => {
                                //Subtract an extra -1 to put it in the middle of the screen (on
                                //screen objects use 0-13, terrain 0-15
                                obj.push(ScreenObject::new(u.get_tile(), (i-1) as u8, (j-1) as u8));
                            },
                            None => {},
                        }
                    } else {
                        ter.push(empty.clone());
                    }
                }
            }
        }
        MapScreen {
            terrain: ter,
            objects:obj,
        }
    }
}
