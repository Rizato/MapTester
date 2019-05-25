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

pub mod camera;
pub mod characters;
pub mod command;
/// This module holds the game object. It has the Game struct
pub mod map;

use glob::glob;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::prelude::*;
use uuid::Uuid;

use self::camera::Camera;
use self::characters::{Character, Npc, Pc};
use self::command::Command;
use self::map::Map;
use conn::Msg;
use conn::{Rx, Tx};

pub struct Game {
    rx: Rx,
    connections: HashMap<SocketAddr, Connection>,
    pcs: HashMap<Uuid, Pc>,
    maps: HashMap<String, Map>,
    mappings: HashMap<String, i16>,
    queue: Vec<Msg>,
}

// This should implement future, because it is not some function used by a future, it is actually the task to be performed
// Whereas poll_login, is a task the future waits on.
impl Game {
    pub fn new(rx: Rx) -> Self {
        let mappings = Game::create_mappings();
        Game {
            rx: rx,
            connections: HashMap::new(),
            pcs: HashMap::new(),
            maps: HashMap::new(),
            mappings: mappings,
            queue: Vec::new(),
        }
    }

    pub fn tick(&mut self) -> Result<(), ()> {
        // Read commands,
        for msg in self.queue.iter() {
            match msg {
                Msg::Login(addr, login) => {
                    let camera = Camera::new(&login.width, &login.height);
                    if let Some(ref connection) = self.connections.get(&addr) {
                        let tx = &connection.tx;
                        // Check if user is logged in w/ a valid conn. -> Fail.
                        let mut exists = false;
                        for (address, existing) in self.connections.iter() {
                            if let Some(ref pc) = self.pcs.get(&existing.id) {
                                if pc.name == login.name {
                                    tx.unbounded_send(Msg::LoginResult(
                                        3,
                                        "User already logged in".to_string(),
                                    ))
                                    .unwrap();
                                    exists = true;
                                }
                            }
                        }
                        // I would have liked a way to exit from the match, instead of this.
                        if !exists {
                            // Check if user is logged in, w/o a valid conn -> Good & takeover pc id
                            for (id, pc) in self.pcs.iter_mut() {
                                if pc.name == login.name {
                                    pc.id = connection.id;
                                    tx.unbounded_send(Msg::LoginResult(4, String::new()))
                                        .unwrap();
                                    exists = true;
                                }
                            }

                            if !exists {
                                // Create new player
                                let player = Pc::new(connection.id.clone(), &login.name, camera);
                                self.pcs.insert(connection.id, player);
                                // TODO Add to lobby
                            }
                            tx.unbounded_send(Msg::TileMapping(self.mappings.clone()));
                        }
                    }
                }
                Msg::Timeout(addr) => {
                    self.connections.remove(&addr);
                }
                Msg::Command(addr, command) => match self.connections.get(&addr) {
                    Some(ref conn) => {
                        let ref id = &conn.id;
                        if let Some(ref pc) = self.pcs.get(id) {
                            pc.add_command(&command);
                        }
                    }
                    None => {
                        error!("Address {:?} does not have an open connection", addr);
                    }
                },
                Msg::Connect(addr, tx) => {
                    let id = Uuid::new_v4();
                    let connection = Connection::new(tx.clone(), id);
                    self.connections.insert(*addr, connection);
                }
                _ => {
                    error!("This command shouldn't be sent to the game");
                }
            }
        }

        for (name, map) in self.maps.iter_mut() {
            let players = &map.players.clone();
            for (id, position) in players {
                if let Some(ref mut player) = self.pcs.get_mut(&id) {
                    let mut commands: Vec<Command> = Vec::new();
                    if let Some(movement) = player.next_movement(&map, &position) {
                        commands.push(movement);
                    }

                    for command in commands {
                        match command {
                            Command::Whisper(target, message) => {}
                            Command::Shout(message) => {
                                for (_, conn) in self.connections.iter() {
                                    if conn.id != *id {
                                        conn.tx
                                            .unbounded_send(Msg::Shout(message.clone()))
                                            .unwrap();
                                    }
                                }
                            }
                            c => {
                                map.queue(c);
                            }
                        }
                    }
                }
            }

            // This gives back commands that require action external to the map (Teleportations & Respawns)
            // TODO Do this as asyncio gather to speed up the process
            let reactions = map.execute();
            for reaction in reactions {
                match reaction {
                    Command::Respawn(Uuid) => {}
                    Command::Teleport(target, map, ask_map, suggested_spot) => {}
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn create_mappings() -> HashMap<String, i16> {
        let mut m: HashMap<String, i16> = HashMap::new();
        let tile_file = File::open("file_full").unwrap();
        let mut reader = BufReader::new(tile_file);
        let mut line: String = String::new();
        let mut count = 0;
        while reader.read_line(&mut line).unwrap() > 0 {
            m.insert(line.clone().trim().to_string(), count.clone());
            count = count + 1;
            line.clear();
        }
        for entry in glob("images/**/*.gif").unwrap() {
            match entry {
                Ok(img) => {
                    m.insert(
                        img.file_stem().unwrap().to_str().unwrap().to_string(),
                        count,
                    );
                    count = count + 1;
                    info!("{} {}", img.display(), count);
                }
                _ => {}
            }
        }
        return m;
    }
}

pub struct GameMessageParser {
    game: Arc<Mutex<Game>>,
    max_messages: u64,
}

impl GameMessageParser {
    pub fn new(game: &Arc<Mutex<Game>>, max_messages: u64) -> Self {
        GameMessageParser {
            game: game.clone(),
            max_messages,
        }
    }
}

impl Future for GameMessageParser {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut game = self.game.lock();
        if let Ok(ref mut game) = self.game.lock() {
            for _x in 0..self.max_messages {
                match game.rx.poll() {
                    Ok(Async::Ready(Some(msg))) => {
                        game.queue.push(msg);
                    }
                    Ok(Async::Ready(None)) => {}
                    Ok(Async::NotReady) => {
                        // Normall, you would just use try_ready! and exit on this.
                        // In our case, we want to be fast, if it isn't ready
                        // It will be for the next loop
                        return Ok(Async::Ready(()));
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Ok(Async::Ready(()))
        } else {
            error!("error getting game lock");
            Err(())
        }
    }
}

pub struct Connection {
    id: Uuid,
    tx: Tx,
}

impl Connection {
    fn new(tx: Tx, id: Uuid) -> Self {
        Connection { tx: tx, id: id }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Point {
    x: u32,
    y: u32,
}

impl Point {
    fn new(x: &u32, y: &u32) -> Self {
        Point {
            x: x.clone(),
            y: y.clone(),
        }
    }

    fn from_index(index: &u32, width: &u32) -> Self {
        let x = index % width;
        let y = index / width;
        Point { x: x, y: y }
    }

    fn to_index(self, width: &u32) -> u32 {
        // Convert x,y to a single index for a 1d array representing a 2d map
        self.y * width
    }
}
