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
use game::characters::ControllableType;
use game::characters::Direction;
use game::characters::player::Player;
use game::characters::item::Item;
use game::characters::connected::RoadWall;
use game::characters::teleporter::Teleporter;
use game::Game;

use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

use xml::reader::{EventReader, XmlEvent};

use self::slab::Index;

/// This module holds all the map related stuff. It has the GameMap itself, along with the
/// MapScreen, ScreenObjects, ScreenTerrain etc.  

///This is the map. It holds all of the terratin, and all of the objects and such. 
///It also holds the x,y of the start value. This is only temporary until I get objects for start
///values
#[derive(Clone)]
pub struct GameMap {
    pub width: u8,
    pub height: u8,
    pub tiles: Arc<Vec<MapTile>>,
    pub objects: Arc<Vec<Box<Controllable>>>,
    pub teleporter: HashMap<u32,Teleporter>,
    start_x: u8,
    start_y: u8, 
}

impl GameMap {
    ///This attemps to parse a file 
    pub fn new(mapname: &str) -> Result<GameMap, String> {
        if !GameMap::maps_exist(mapname) {
            return Err("Map Not Found".to_string());
        }
        GameMap::parse_tiles(mapname)
    }
    
    ///Checks to see if the map exists
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

    /// This is without a doubt the most hideos part of the code, and there are a lot of hideous
    ///parts.
    ///Basically, it opens the xml map file and parses it out. There are a few special sections that
    ///it handles. Header, Terrain, Roads and Teleporters. 
    fn parse_tiles(path: &str) -> Result<GameMap, String>{
        println!("Parsing! {}", path);
        match File::open(path) {
            Err(_) => {
               Err("Failed to parse".to_string()) 
            },
            Ok(file) => {
                //The map struct variables
                let mut tiles: Vec<MapTile> = vec![];
                let mut objects: Vec<Box<Controllable>> = vec![];
                let mut teleporters: HashMap<u32, Teleporter>= HashMap::new();
                let mut width: u8 = 0;
                let mut height: u8 = 0;
                let mut start_x: u8 = 0;
                let mut start_y: u8 = 0;
                
                //Declaring a bunch of variables to temporarily store values that cannot
                //be turned into structs until an object closes.
                
                //Because headers & teleporters are not complete until the object closes, they store the 
                //required children values in these variables.
                
                //header
                let mut header = false;
                let mut out_of_bounds_tile = String::new();
                
                //teleporter
                let mut teleporter = false;
                let mut teleporter_index = 0;
                let mut teleporter_use_default = false;
                let mut teleporter_x = 0;
                let mut teleporter_y = 0;
                let mut teleporter_height = 1;
                let mut teleporter_width = 1;
                let mut teleporter_map = String::new();
                
                //Values needed for the parser
                let tile_mappings = Game::create_mappings();
                let buf = BufReader::new(file);
                let parser = EventReader::new(buf);
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
                            } else if name.local_name == "bean" {
                                //We don't do anything with this
                            } else if name.local_name == "boolean" {
                                if teleporter {
                                    for attr in attributes {
                                        if attr.name.local_name == "name" 
                                            && attr.value == "ask-map" {
                                            teleporter_use_default = true;
                                        }
                                    }
                                }
                            } else if name.local_name == "string" {
                                let mut is_map = false;
                                let mut name = String::new();
                                if teleporter {
                                    for attr in attributes {
                                        if attr.name.local_name == "name" 
                                            && attr.value =="map" {
                                                is_map = true;
                                        } else if attr.name.local_name == "value" {
                                            name = attr.value.to_string();
                                        }
                                    }
                                    if is_map {
                                        teleporter_map = name;
                                    }
                                }
                            } else if name.local_name == "arch" {
                                if header {
                                    let mut terrain = String::new();
                                    let mut is_terrain = false;
                                    let mut is_oob = false;
                                    for attr in attributes {
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
                                        for _ in 0..size {
                                            let tile = MapTile::new(terrain.clone());
                                            tiles.push(tile);
                                        }
                                    } else if is_oob {
                                        out_of_bounds_tile = terrain;
                                    }
                                } else {
                                    //Read the loc. 
                                    //If it is a rectangle, apply that tile to the
                                    //  entire area as the terrain.
                                    //Else add it to the array of map items  (need to refactor maps)
                                    let mut tile: String = "".to_string();
                                    let mut rect_x: u8 = 0;
                                    let mut rect_y: u8 = 0;
                                    let mut rect_w: u8 = 0;
                                    let mut rect_h: u8 = 0;
                                    for attr in attributes {
                                        if attr.name.local_name == "path" {
                                            tile= attr.value;
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
                                    if tile.contains("terrain") {
                                        for x in rect_x..(rect_x+rect_w) {
                                            for y in rect_y..(rect_y+rect_h) {
                                                let index: usize = (y as usize * width as usize) as usize + x as usize;
                                                tiles[index].tile = tile.clone();
                                                tiles[index].blocked = false;
                                            }
                                        }
                                    } else if tile == "special/teleporter".to_string() {
                                        println!("Started teleporter");
                                        let index: u32 = rect_y as u32 * width as u32 + rect_x as u32;
                                        teleporter = true;
                                        teleporter_index = index.clone();
                                        teleporter_height = rect_h.clone();
                                        teleporter_width = rect_w.clone();
                                        println!("Finished teleporter");
                                    } else {
                                        for x in rect_x..(rect_x+rect_w) {
                                            for y in rect_y..(rect_y+rect_h) {
                                                let index: u32 = (y as u32 * width as u32 ) as u32 + x as u32;
                                                //This is a special case. The map editor treats the
                                                //main road as terrain
                                                if tile.contains("main_road") {
                                                    tiles[index as usize].blocked = false;
                                                }
                                                //TODO doors & windows
                                                if tile.contains("roads") || tile.contains("walls") { 
                                                    objects.push(Box::new(RoadWall::new(tile.clone(), &tile_mappings, index)));
                                                } else { 
                                                    objects.push(Box::new(Item::new(tile.clone(), index)));
                                                }
                                            }
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
                                if teleporter && name == "x" {
                                    teleporter_x = val;
                                } else if teleporter && name == "y"{
                                    teleporter_y = val;
                                } else if name.contains("startX") {
                                    start_x = val;
                                } else if name.contains("startY") {
                                    start_y = val;
                                }
                            }
                        },
                        Ok(XmlEvent::EndElement {name}) => {
                            if name.local_name == "header" {
                                header = false;
                                for index in 0..tiles.len() {
                                    if tiles[index].tile == out_of_bounds_tile {
                                        tiles[index].blocked = true;
                                    }
                                }
                            } else if name.local_name == "arch" && teleporter {
                                println!("teleporter: {}", teleporter_index);
                                let t_x = teleporter_index % width as u32;
                                let t_y = teleporter_index / width as u32;
                                for x in 0..teleporter_width {
                                    for y in 0..teleporter_height {
                                        println!("map {} index {} default {}", teleporter_map,
                                                 teleporter_index, teleporter_use_default);
                                        let index = (t_y as u32 + y as u32) as u32 * width as u32 + (t_x as u32 +x as u32) as u32;
                                        teleporters.insert(teleporter_index,
                                                           Teleporter::new(teleporter_map.clone(),
                                                                              index,
                                                                              teleporter_use_default,
                                                                              teleporter_x,
                                                                              teleporter_y));
                                    }
                                }

                                //Reset teleporter values
                                teleporter = false;
                                teleporter_index = 0;
                                teleporter_use_default = false;
                                teleporter_x = 1;
                                teleporter_y = 1;
                                teleporter_height = 1;
                                teleporter_width = 1;
                                teleporter_map ="".to_string();
                            }
                        },
                        _ => {

                        },
                    }
                }
                //Telling roads & walls to draw based on surrounding tiles
                let mut roads = vec![false; tiles.len()];
                let mut wall = vec![false; tiles.len()];
                let len = objects.len();
                for i in 0..len {
                    let ref mut object = objects[i];
                    match object.get_type() {
                        ControllableType::Road => {
                            roads[object.get_location() as usize] = true;
                        },
                        ControllableType::Wall => {
                            wall[object.get_location() as usize] = true;
                        },
                        _ => {},
                    }
                }
                for i in 0..len {
                    let ref mut object = objects[i];
                    match object.get_type() {
                        ControllableType::Road => {
                            object.modify_connected_tiles(width, height, &roads);
                        },
                        ControllableType::Wall=>{
                            object.modify_connected_tiles(width, height, &wall);
                        },
                        _ => {},
                    }
                }
                Ok(GameMap{
                    width: width,
                    height: height,
                    tiles: Arc::new(tiles),
                    objects: Arc::new(objects),
                    teleporter: teleporters,
                    start_x : start_x,
                    start_y : start_y,
                })
            },
        }
    }

    /// Adds the command from the client to the user object
    pub fn push_command(&mut self, token: mio::Token, command: String) {
        println!("push command");
        match self.find_player_with_token(token.clone()) {
            Some(index) => {
                match Arc::get_mut(&mut self.objects) {
                    Some(objects) => {
                        match objects.get_mut(index) {
                            Some(ref mut p) => {
                                println!("Command {}", command);
                                if command.starts_with("mouse") {
                                    let parts: Vec<&str> = command.split_whitespace().collect();
                                    //Mouse click x,y
                                    let mx = parts[1].parse::<i32>().unwrap();
                                    let my = parts[2].parse::<i32>().unwrap();
                                    //old x,y
                                    let oy = p.get_location() / (self.width as u32);
                                    let ox = p.get_location() % (self.width as u32);
                                    //change in x,y. -6 cause user is always in middle of screen, no matter the click.
                                    let dx = if ox as i32 + mx > 6 { ox + mx as u32 -6 } else {0};
                                    let dy = if oy as i32 + my > 6 { oy + my as u32 -6 } else {0};
                                    println!("Move to x{} y{}", dx, dy);
                                    let end = dy * self.width as u32 + dx;
                                    //tiles[start as usize].user.unwrap().set_movement(end.clone());
                                    p.set_movement(end.clone());
                                } else {
                                    p.push_command(command);
                                }
                            },
                            None => {},
                        }
                    },
                    None => {},
                }
           },
           None => {}
        }
    }

    /// Returns the x,y value of a token
    fn find_player_with_token(&self, token: mio::Token) -> Option<usize> {
        let ref objects= self.objects;
        let len = objects.len();
        for i in 0..len {
            let ref object = objects[i];
            match object.get_token() {
                Some(ref tok) => {
                    if tok.as_usize() == token.as_usize() {
                        return Some(i);
                    }
                },
                None =>{},
            }
        }
        None
    }
    
    /// This goes through all connections, tries to read off the queue, and then executes each
    ///command, possibly returning a tailored response 
    pub fn execute(&mut self) -> Vec<(mio::Token, u8, String)> {
        let mut retval = vec![];
        //Looping through all tiles
        let width = self.width;
        let height = self.height;
        let mut blocked: Vec<bool> = vec![false; self.tiles.len()];
        match Arc::get_mut(&mut self.objects) {
            Some(ref mut objects) => {
                let len = objects.len();
                for i in 0..len {
                    //O(n^2 * m) to find blockages
                    for i in 0..len {
                        let ref o = objects[i];
                        let index = o.get_location();
                        if !blocked[index as usize] {
                            blocked[index as usize] = o.does_block_index(index);
                        }
                    }
                    let tiles_len = self.tiles.len();
                    for i in 0..tiles_len {
                        if !blocked[i] {
                            blocked[i] = self.tiles[i].blocked;
                        }
                    }
                    //I hate that I generate a list of blocked tiles. I would rather 
                    //pass a reference to the map objects, but I could not figure
                    //out how to do that in rust, since it does not allow any
                    //immutable borrows if there is a mutable borrow & the update
                    //function itself requires a mutable borrow
                    match objects[i].update(width, height, &blocked) {
                        Some(responses) => {
                                for x in 0..responses.len() {
                                    let (token, style, response) = responses[x].clone();
                                    retval.push((token, style, response));
                                }
                        },
                        None => {},
                    }
                }
            },
            None =>{
                println!("Got None"); 
            },
        }
        retval
    }

    ///Checks for any players on teleporters. Returns a vector of tuples containing
    ///the player token, the map name and a start x,y if specified by the teleporter
    pub fn do_teleports(&self) -> Vec<(mio::Token, String, Option<(u8,u8)>)> {
        let mut retval = vec![];
        let ref objects = self.objects;
        let len = objects.len();
        for i in 0..len {
            let ref index = objects[i].get_location();
            match objects[i].get_token() {
                Some(token) => {
                    match self.teleporter.get(index) {
                        Some(tele) => {
                            retval.push(tele.teleport(token.clone()))
                        },
                        None => {},
                    }
                },
                None => {},
            }
        }
        retval
    }

    
    
    /// This pulls the HP from health. 
    pub fn get_hp(&self, token: mio::Token) -> Option<i32> {
        match self.find_player_with_token(token.clone()) {
            Some(index) => {
                match self.objects.get(index) {
                    Some(ref p) => {
                        p.get_hp()
                    },
                    None => {
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
    pub fn send_portion(&self,token: mio::Token) -> Option<MapScreen> {
        //println!("Send Portion");
        //This sends the squares around the user, which will always be centered in the screen.
        match self.find_player_with_token(token.clone()) {
            Some(index) => {
                match self.objects.get(index) {
                    Some(ref p) => {
                        let x = p.get_location() % self.width as u32;
                        let y = p.get_location() / self.width as u32;
                        let (view_x, view_y) = p.get_viewport();
                        Some(MapScreen::new(self, x, y, view_x, view_y))
                    },
                    None => {
                        return None;
                    },
                }
            },
            None => {
                return None;
            },
        }
    }

    /// Adds a player to the map. Puts it at the starting location.
    pub fn add_player(&mut self, token: mio::Token, name:String, index: Option<(u8, u8)>) {
        println!("Add Player");
        let startx;
        let starty;
        match index {
            Some((x,y)) => {
                startx = x;
                starty = y;
            },
            None => {
                startx = self.start_x;
                starty = self.start_y;
            },
        }
        self.add_player_at(&mut Player::new(name, token), startx, starty, Direction::All);
    }

    ///Recursively searches for an open location. Sadly this is a horrible algorithm, and will build characters
    ///all in one direction before it tries any of the others. Ugly. What really kills it is that
    ///it exectures on the game loop & stops everything while it searches. If the x direction is
    ///full, it will take forever.
    fn add_player_at(&mut self, player: &mut Player, x: u8, y: u8, direction: Direction) -> bool {
        println!("adding at {} {}", x, y);
        let index = y as u32 * self.width as u32 + x as u32;
        if x < self.width && y < self.height {
            let mut is_open = true;
            {
                match Arc::get_mut(&mut self.objects) {
                    Some(objects) => {
                        for i in 0..objects.len() {
                            if objects[i].does_block_index(index) {
                                is_open = false;
                                break;
                            }
                        }
                    },
                    None => {},
                }
            }
            if is_open {
                player.set_location(index);
                match Arc::get_mut(&mut self.objects) {
                    Some(objects) => {
                        objects.push(Box::new(player.clone()));
                    },
                    None =>{},
                }
                true
            } else {
                match direction {
                     Direction::All => { 
                         (x > 0 && self.add_player_at(player, x-1, y, Direction::West))
                             ||  self.add_player_at(player, x+1, y, Direction::East) 
                             || (y > 0 && self.add_player_at(player, x, y-1, Direction::North))
                             || self.add_player_at(player, x, y+1, Direction::South)
                     },
                     Direction::East=> { 
                          self.add_player_at(player, x + 1, y, Direction::East)
                     },
                     Direction::West=> { 
                          x > 0 && self.add_player_at(player, x - 1, y, Direction::West)
                         
                     },
                     Direction::South=> { 
                          self.add_player_at(player, x, y+1, Direction::South)
                     },
                     Direction::North=> { 
                         y > 0 && self.add_player_at(player, x, y-1, Direction::North)
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
        match Arc::get_mut(&mut self.objects) {
            Some(objects) => {
                let len = objects.len();
                let mut remove_index = None;
                for i in 0..len {
                    let ref object = objects[i];
                    match object.get_token() {
                        Some(ref tok) => {
                            if tok.as_usize() == token.as_usize() {
                                remove_index = Some(i);

                            }
                        },
                        None =>{},
                    }
                }
                match remove_index {
                    Some(index) => {
                        objects.remove(index);
                    },
                    None => {},
                }
            },
            None => {},
        }
        println!("FInished remove");
    }
}

/// A single tile option, which optionally holds a user. Holds an image tile, and whther the tile
/// is blocked or not.
#[derive(Clone)]
pub struct MapTile{
    //No position, because position is determined by the position in vector
    pub tile: String,
    pub blocked: bool,
}

impl MapTile {
    fn new(tile: String) -> MapTile {
        MapTile{
            tile: tile,
            blocked: false,
        }
    }
}

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
            tile: MapScreen::convert_terrain(tile),
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
            tile: MapScreen::convert_terrain(tile),
        }
    }

    ///To get fancy borders the tile has to have a priority assigned. I have
    ///hardcoded some values here.
    pub fn get_priority(&self) -> u32 {
        let priority;
        if self.tile.contains("grass") {
            priority = (2 as u32) << 17; 
        } else if self.tile.contains("shallow") {
            priority = (1 as u32) << 17;
        } else if self.tile.contains("water") {
            priority = (3 as u32) << 17;
        } else if self.tile.contains("trees") 
            || self.tile.contains("forest") 
            || self.tile.contains("wall") {
            priority = (4 as u32) << 17;
        } else if self.tile.contains("lava") {
            priority = (10000 as u32) << 17;
        } else {
            priority = (0 as u32) << 17;
        }
        priority | (7 as u32) << 29
    }
}

///Holds a vector of terrain objects, and a series of objects. The terrain is just a vector,
///representing a 15x15 matrix. The screen objects on the other hand denote their x and y
///specifically, rather than by their position in the array.
#[derive(Clone)]
pub struct MapScreen {
    pub width: i16,
    pub height: i16,
    //15x15 vector. 
    pub terrain: Vec<ScreenTerrain>,
    //User art at 7,7
    pub objects: Vec<ScreenObject>,
}

impl MapScreen {
    ///generates a new MapScreen based on the map and a given x & y. This will grab the 15x15
    ///matrix centered on the given x and y. Any spaces beyond the boundaries of the map is replaced
    ///with "empty" tiles
    pub fn new(map: &GameMap, x: u32, y: u32, size_x: u8, size_y: u8) -> MapScreen {
        let startx: isize = x as isize -(size_x as isize /2 as isize + 1);
        let starty: isize = y as isize -(size_y as isize /2 as isize + 1);
        let mut ter = Vec::with_capacity((size_x+2) as usize *(size_y+2) as usize);
        let mut obj = vec![];
        //If coords are valid we will actually draw something
        let empty = ScreenTerrain::new("terrain/empty".to_string());
        //creates array of tiles
        if map.width as u32 > x && map.height as u32 > y {
            for i in 0..(size_x as isize+2) {
                for j in 0..(size_y as isize+2) {
                    if startx+i >= 0 && startx+(i as isize) < (map.width as isize) && starty+(j as isize) >= 0 && starty+(j as isize) < (map.height as isize) {
                        //get the tile from the map
                        let index= ((starty +j) * (map.width as isize)+ (startx+i)) as usize;
                        let ref tiles = map.tiles;
                        //clone the map tile
                        let tile = tiles[index as usize].clone();
                        //Add the terrain from the tile
                        ter.push(ScreenTerrain::new(tile.tile.clone()));
                    } else {
                        ter.push(empty.clone());
                    }
                }
            }
        }
        //Creates array of objects
        let ref objects = map.objects; 
        for i in 0..objects.len() {
            let ref object = objects[i];
            let index = object.get_location();
            let object_x = index % map.width as u32;
            let object_y = index / map.width as u32;
            if object_x as isize >= startx && object_x < map.width as u32 && object_y as isize >= starty
                && object_y < map.height as u32  && object.is_visible(map) {
                    //Extra -1 is to account for the extra tile off screen.
                obj.push(ScreenObject::new(object.get_tile(), (object_x as isize - startx -1) as u8 , (object_y as isize - starty -1) as u8));
            }
        }
        MapScreen {
            width: size_x as i16 +2,
            height: size_y as i16 +2,
            terrain: ter,
            objects: obj,
        }
    }

    ///These are the terrain tiles I have found where the terrain path from xml
    ///did not match the image path.
    fn convert_terrain(tile: String) -> String {
        if tile == "scenery/sign1" {
            "indoor/sign1".to_string()
        } else if tile == "scenery/caveE" {
            "scenery/cave2".to_string()
        } else if tile == "terrain/dark_earth" {
            "terrain/mud".to_string()
        } else {
            tile
        }
    }

}
