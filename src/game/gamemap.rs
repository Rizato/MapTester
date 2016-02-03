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

use game::characters::Moveable;
use game::characters::player::Player;

use std::sync::RwLock;
use std::sync::Arc;

/// This module holds all the map related stuff. It has the GameMap itself, along with the
/// MapScreen, ScreenObjects, 

/// Enum for the direction that a moveable object just went. Gets sent to the connection when
/// deciding what tile to draw.
#[derive(Clone)]
enum Direction {
    North,
    South,
    East,
    West,
}

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
}

impl GameMap {
    /// This currently creates a generic map. It will eventually load a map by filename and turn
    /// that into a valid MapObject
    pub fn new(mapname: &str) -> Result<GameMap, &str> {
        //TODO Load map from file use ProtocolBuffers.
        let mut tiles: Vec<MapTile> = vec![];
        //Just doing a fake thing really quick.
        for _ in 0..100 {
            tiles.push(MapTile::new("terrain/aspens".to_string()));
        }
        for _ in 0..100 {
            tiles.push(MapTile::new("terrain/beach".to_string()));
        }
        for _ in 0..100 {
            tiles.push(MapTile::new("terrain/blue_tile".to_string()));
        }
        for _ in 0..100 {
            tiles.push(MapTile::new("terrain/brick1".to_string()));
        }
        for _ in 0..100 {
            tiles.push(MapTile::new("terrain/brick2".to_string()));
        }
        for _ in 0..100 {
            tiles.push(MapTile::new("terrain/brick3".to_string()));
        }
        for _ in 0..100 {
            tiles.push(MapTile::new("terrain/brick4".to_string()));
        }
        for _ in 0..100 {
            tiles.push(MapTile::new("terrain/carpet1".to_string()));
        }
        for _ in 0..100 {
            tiles.push(MapTile::new("terrain/carpet2".to_string()));
        }
        let mut ti = Arc::new(RwLock::new(tiles));
        let map = GameMap {
            width: 30,
            height: 30,
            start_x: 15,
            start_y: 8,
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
        let mut tiles = self.tiles.read().unwrap();
        tiles[index as usize].clone()
    }

    ///Moves a user from one tile to another, replaces the direction witht he given direction
    fn move_user(&mut self, o:u32, n:u32, d: Direction) -> bool {
        //println!("{}", n);
        let old = self.get_user(o); 
        let mut tiles = self.tiles.write().unwrap();
        let ref mut new = tiles[n as usize];
        match new.user {
            Some(_) => {
                false
            },
            None => {
                let mut u = old.user.clone().unwrap();
                u.clear_movement_if_at_destination(n);
                u.direction = d.clone();
                new.user = Some(u);
                new.blocked = true;
                true
            }
        }
    }

    /// Removes the tile at the given index
    fn wipe_user(&mut self, o: u32) {
        //TODO make sure the tile was not empty before (Cause if it was empty and was blocked it is
        //blocked by an object, and we don't want to unblock.
        let mut tiles = self.tiles.write().unwrap();
        let ref mut old = tiles[o as usize];
        old.user = None;
        old.blocked = false;
    }

    /// Adds the command from the client to the user object
    pub fn push_command(&mut self, token: mio::Token, command: String) {
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
                    if u.token == token {
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
        let mut retval = vec![];
        for token in conns.iter() {
            let (x, y) = self.find_tile_with_token(token.clone()).unwrap();
            let index = y * self.width as u32 + x;
            let mut command = None;
            let mut t= None;
            {
                let mut tile = self.tiles.write().unwrap();
                command = match tile[index as usize].user {
                    Some(ref mut u) => { 
                        t = Some(u.token.clone());
                        u.get_command() 
                    },
                    None => None,
                };
            }
            match command {
                Some(c) => {
                    match self.execute_command(t.unwrap(), c) {
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
    fn execute_command(&mut self, token: mio::Token, command: String ) -> Option<Vec<(mio::Token, u8, String)>> {
        let (x, y) = self.find_tile_with_token(token.clone()).unwrap();
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
                    Some(vec![(token.clone(), 5,  "No Path Found".to_string()); 1])
                },
            }
        } else {
            //System message
            println!("{}", command);
            Some(vec![(token.clone(), 5,  "Bad command".to_string()); 1])
        }
    }
    
    /// This generates a new MapScreen based on the location of the given connection's user
    pub fn send_portion(&self, token: mio::Token) -> MapScreen {
        //This sends the squares around the user, which will always be centered in the screen.
        let (x, y) = self.find_tile_with_token(token.clone()).unwrap();
        MapScreen::new(self, x, y)
    }

    /// Adds a player to the map. Puts it at the starting location.
    pub fn add_player(&mut self, token: mio::Token, player: Arc<Player>) {
        //TODO Add match start.user None/Some & determine whether to add in a different location
        let mut tiles = self.tiles.write().unwrap();
        let ref mut start = tiles[(self.start_y * self.width + self.start_x) as usize];
        start.user = Some(MapUser::new(token.clone(), player.clone()));
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
    //TODO add a Vec<MapItem>
}

impl MapTile {
    fn new(tile: String) -> MapTile {
        MapTile{
            tile: tile,
            user: None,
            blocked: false,
        }
    }
}

///Is a controllable thing on the map. Has a tile, which does not hold a direction(to be added
///later). It also holds a queue of directions, and the player object at the location. Lastly, it
///holds some movement helper values. That is, the final destination of the user initiated
///movement, as well as the number of cycles since the last move. This is used in conjunction witht
///he player Speed value.
#[derive(Clone)]
pub struct MapUser{
    player: Arc<Player>,
    tile: String,
    token: mio::Token,
    commands: Vec<String>,
    direction: Direction,
    //Coordinates (x, y). This is where the char is currently trying to move to. It has to be interpereted by the Map, and converted to an x, y relative to the actual map, not the user.
    movement: Option<u32>,
    movement_ticks: u8,
     
}

impl  MapUser {
    ///This creates a new map user object, with some defaults. Takes in a player object and the
    ///token for the connection
    fn new(token: mio::Token, player: Arc<Player>) -> MapUser {
       MapUser {
            token: token, 
            player: player.clone(),
            tile: player.tile.clone(),
            commands: vec![],
            direction: Direction::South,
            movement: None,
            movement_ticks: 0,
       }
    }
    
    ///Pushes a command to the command queue for the mapuser
    fn push_command(&mut self, command: String) {
        self.commands.insert(0, command);
    }
    
    ///Puts an absolute X, Y as the movement goal for this mapuser
    fn set_movement(&mut self, end: u32) {
        self.movement = Some(end);
    }

    ///Checks the end index against the movement goal index. If they are the same,
    /// it wipes out the movement goalt.
    fn clear_movement_if_at_destination(&mut self, end: u32) {
        let replacement: Option<u32>  = match self.movement {
            Some(e) => {
                if e == end {
                   None
                } else {
                    Some(e)
                }
            },
            None => {None},
        };
        self.movement = replacement;
    }
    
    ///Grabs an available command from the queue. Manages movement by counting the number of cycles
    ///since last movment. This prefers movement over the top of the queue. SO basically
    /// 
    /// This checks the number of ticks, against a threshold. If it is greater or equal, and there is a
    ///movement goal set, it will always do movement. Otherwise, it will increment ticks, and grab
    ///the top command from the queue.
    fn get_command(&mut self) -> Option<String> {
        let has_movement = match self.movement{
            Some(_) =>  {true},
            None => {false},
        };
        if has_movement && self.movement_ticks >= self.player.speed /* *self.slow */ {
            //The command returns the absolute location where the user wants to end up. The map knows it can only move 1 space towards that destination
            let end = self.movement.unwrap();
             self.movement_ticks = 0;
             println!("got command {}", end);
            Some(format!("end {}", end))
        } else if self.commands.len() > 0 {
            self.movement_ticks = if self.movement_ticks == 255 {
                self.movement_ticks 
            } else {
                self.movement_ticks + 1
            };
            self.commands.pop()
        } else {
            self.movement_ticks = if self.movement_ticks == 255 {
                self.movement_ticks 
            } else {
                self.movement_ticks + 1
            };
            None
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
                                let mut t_with_d = u.tile.clone();
                                t_with_d.push_str(match u.direction {
                                    Direction::South => {"S"},
                                    Direction::North => {"N"},
                                    Direction::East => {"E"},
                                    Direction::West => {"W"},
                                });
                                obj.push(ScreenObject::new(t_with_d.clone(), (i-1) as u8, (j-1) as u8));
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
