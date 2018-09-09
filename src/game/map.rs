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
limitations under the License. */

use game::characters::{Character, Pc, Npc};
use game::command::{Command, Direction};
use game::{Game, Point};

use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use uuid::Uuid;


use xml::reader::{EventReader, XmlEvent};

/// This module holds all the map related stuff. It has the GameMap itself, along with the
/// MapScreen, ScreenObjects, ScreenTerrain etc.

///This is the map. It holds all of the terratin, and all of the objects and such.
///It also holds the x,y of the start value. This is only temporary until I get objects for start
///values
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub modifiers: HashMap<String, String>, // Things like lighting, nopk, start, oob terrain, etc.
    pub tiles: Vec<MapTile>, // A vector a tiles
    pub players: HashMap<Uuid, Point>,
    pub objects: Vec<Npc>, // (Teleporters are now objects) A vec of objects on the map. Each is a character so it can be controlle
    pub queue: Vec<Command>, // Queue of player commands to execute
}

impl Map {
    ///This attemps to parse a file
    pub fn new(mapname: &str) -> Self {
        Map {
            width: 0,
            height: 0,
            modifiers: HashMap::new(),
            tiles: Vec::new(),
            players: HashMap::new(),
            objects: Vec::new(),
            queue: Vec::new(),
        }
    }

    pub fn add_player(&mut self, uuid: &Uuid) {
        let x = self.modifiers.get("start_x").unwrap().parse::<u32>().unwrap();
        let y = self.modifiers.get("start_y").unwrap().parse::<u32>().unwrap();
        self.players.insert(uuid.clone(), Point::new(&x, &y));
    }

    /// Returns the x,y value of a token
    pub fn find_player(&mut self, uuid: &Uuid) -> Option<Point> {
        let ref objects= self.objects;
        let len = objects.len();
        if let Some(location) = self.players.get(uuid) {
            return Some(location.clone());
        }

        None
    }

    pub fn remove_player(&mut self, uuid: &Uuid) {
        self.players.remove(uuid);
    }

    pub fn queue(&mut self, command: Command) {
        self.queue.push(command);
    }

    /// This goes through all connections, tries to read off the queue, and then executes each
    ///command, possibly returning a tailored response
    pub fn execute(&mut self) -> Vec<Command> {
        let mut results = vec![];
        for command in &self.queue {
            match command {
                Command::MoveStep(ref uuid, ref point, ref direction) => {
                    self.players.insert(uuid.clone(), point.clone());
                    // How do I handle the direction?
                },
                _ => {},
            };
        }

        let mut player_locations: Vec<Point> = Vec::new();
        for (_, point) in self.players.iter() {
            player_locations.push(point.clone());
        }

        for npc in &self.objects {
            if player_locations.contains(&npc.location) {
                for (uuid, point) in self.players.iter() {
                    if point.clone() == npc.location {
                        if let Some(ref delegate) = &npc.delegate {
                            if let Some(callback) = delegate.when_activated {
                                callback(&uuid);
                            }
                        }
                    }
                }
            }
        }

        results
    }
}

/// A single tile option, which optionally holds a user. Holds an image tile, and whther the tile
/// is blocked or not.
#[derive(Clone)]
pub struct MapTile {
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
