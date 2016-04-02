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

use game::characters::Controllable;
use game::characters::Direction;
use game::characters::player::Player;
use game::characters::tower::Tower;
use game::characters::projectile::Projectile;

use std::sync::RwLock;
use std::sync::Arc;
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
    //TODO This is temporary
    start_x: u8,
    start_y: u8, 
    tower_index: u32,
}

impl GameMap {
    /// This currently creates a generic map. It will eventually load a map by filename and turn
    /// that into a valid MapObject
    pub fn new(mapname: &str) -> Result<GameMap, &str> {
        //TODO Load map from file use ProtocolBuffers.
        let mut tiles: Vec<MapTile> = vec![];
        //Just doing a fake thing really quick.
        for _ in 0..360 {
            tiles.push(MapTile::new("terrain/grass".to_string()));
        }
        for _ in 0..5 {
            for _ in 0..4 {
                tiles.push(MapTile::new("terrain/gray_brick".to_string()));
            }
            for _ in 0..22 {
                tiles.push(MapTile::new("terrain/dirt".to_string()));
            }
            for _ in 0..4 {
                tiles.push(MapTile::new("terrain/gray_brick".to_string()));
            }
        }
        for _ in 0..390 {
            tiles.push(MapTile::new("terrain/grass".to_string()));
        }
        
        let t_index = 405;
        tiles[t_index as usize].user = Some(MapUser::new(None,Commandable::T(Tower::new())));
        tiles[t_index as usize].blocked = true;
        let mut ti = Arc::new(RwLock::new(tiles));
        let map = GameMap {
            width: 30,
            height: 30,
            start_x: 29,
            start_y: 14,
            tower_index: t_index,
            
            //Coordinates in tiles will simulate a 2d matrix, while actually being a 1d array.
            // Everything will be found by multiplying the width * y + x
            //   0  1  2  3  4  5  6  7
            // 0 0  1  2  3  4  5  6  7
            // 1 8  9  10 11 12 13 14 15
            tiles: ti.clone(), 
        };
        Ok(map)
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
        println!("Finding Tiles");
        let tiles = self.tiles.read().unwrap();
        println!("Found Tiles!");
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
        let x  = self.tower_index % self.width as u32;
        let y  = self.tower_index / self.width as u32;
        
        //Pushing all tokens to the tower   
        {
            let mut players= vec![];
            //Grabbing all of the players around the tower index
            let mut tiles = self.tiles.write().unwrap();
            let start_x = if x > 8 { x-8} else {0};
            let end_x = if x +10 < self.width as u32 {x + 10} else {self.width as u32};
            let start_y = if y > 8 { y-8} else {0};
            let end_y = if y +10 < self.height  as u32 {y + 10} else {self.height as u32};
            for i in start_x..end_x {
                for j in start_y..end_y {
                    let ref player = tiles[j  as usize * self.width as usize + i as usize];
                    match player.user {
                       Some(ref u) => {
                           match u.token {
                               Some(ref t) => {
                                   &players.push(t.clone());
                               },
                               _ => {},
                           }
                        }, 
                        _ => {},
                    }
                }
            }
            match tiles[self.tower_index as usize].user {
                Some(ref mut user) => {
                    match user.player {
                        Commandable::T(ref mut tower) => {
                            tower.push_tokens(players); 
                        },
                        _ => {},
                    } 
                },
                _ => {},
            };
        }
        let mut retval = vec![];
        //Looping through all tiles
        for i in 0..900 {
            let mut t = None;
            let mut command = None;
            let mut projectile_command = None;
            {
                //Gets the command the mapuser on this tile (if one exists)
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
                //gets the command for a projectile on this tile, if one exists
                projectile_command = match tiles[i].projectile {
                    Some(ref mut p) => {
                        p.get_command(i as u32)
                    },
                    _ => {None},
                };
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
            //executing command from projectile
            match projectile_command {
                Some(c) => {
                    println!("command {}", c);
                    match self.execute_command(None, c) {
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
        }
        retval
    }
    
    ///Executes a given command. Generates a possibly generates a vector of responses.
    fn execute_command(&mut self, token: Option<mio::Token>, command: String ) -> Option<Vec<(mio::Token, u8, String)>> {
        println!("Execute Command");
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
                } else {
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
        println!("Send Portion");
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
    tile: String,
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
        let mut ter = vec![];
        let mut obj = vec![];
        //If coords are valid we will actually draw something
        let empty = ScreenTerrain::new("terrain/empty".to_string());
        if map.width as u32 > x && map.height as u32 > y {
            for i in 0..15{
                for j in 0..15{
                    if startx+i >= 0 && startx+(i as isize) < (map.width as isize) && starty+(j as isize) >=0 && starty+(j as isize) < (map.height as isize) {
                        //get the tile from the map
                        let index= ((starty +j) * (map.width as isize)+ (startx+i)) as usize;
                        let tiles = map.tiles.read().unwrap();
                        //clone the map tile
                        let tile = tiles[index as usize].clone();
                        //Add the terrain from the tile
                        ter.push(ScreenTerrain::new(tile.tile.clone()));
                        match tile.user {
                            Some(u) => {
                                obj.push(ScreenObject::new(u.get_tile(), (i-1) as u8, (j-1) as u8));
                            },
                            None => {},
                        }
                        match tile.projectile {
                            Some(p) => {
                                obj.push(ScreenObject::new(p.get_tile(), (i-1) as u8, (j-1) as u8));
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
