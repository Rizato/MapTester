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

/// This module holds the game object. It has the Game struct

// pub mod gameloop;
// pub mod gamemap;
// pub mod characters;

use glob::glob;
use std::io::prelude::*;
use std::io::Error;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use uuid::Uuid;
use tokio::prelude::*;

use conn::Msg;
use conn::{Tx, Rx};

pub struct Game {
    rx: Rx,
    connections: HashMap<SocketAddr, Connection>,
    pcs: HashMap<Uuid, Pc>,
    npcs: Vec<Npc>,
    maps: HashMap<String, Map>,
}

// This should implement future, because it is not some function used by a future, it is actually the task to be performed
// Whereas poll_login, is a task the future waits on.
impl Game {
    pub fn new(rx: Rx) -> Self {
        Game {
            rx: rx,
            connections: HashMap::new(),
            pcs: HashMap::new(),
            npcs: Vec::new(),
            maps: HashMap::new(),
        }
    }

    pub fn poll(&mut self) -> Poll<(), Error> {
        const COMMANDS_PER_TICK: usize = 50;
        // Read commands,
        for _x in 0..COMMANDS_PER_TICK {
            if let Ok(Async::Ready(Some(msg))) = self.rx.poll() {
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
                                        tx.send(Msg::LoginResult(3, "User already logged in".to_string())).unwrap();
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
                                        tx.send(Msg::LoginResult(4, String::new())).unwrap();
                                        exists = true;
                                    }
                                }

                                if !exists {
                                    // Create new player
                                    let player = Pc::new(connection.id.clone(), &login.name, camera);
                                    self.pcs.insert(connection.id, player);
                                    // TODO Add to lobby
                                }

                            }
                        }
                    },
                    Msg::Timeout(addr) => {
                        self.connections.remove(&addr);
                    },
                    Msg::Command(addr, command) => {
                        match self.connections.get(&addr) {
                            Some(ref conn) => {
                                let ref id = &conn.id;
                                if let Some(ref pc) = self.pcs.get(id) {
                                    pc.add_command(&command);
                                }
                            },
                            None => {
                                println!("Address {:?} does not have an open connection", addr);
                            },
                        }
                    },
                    Msg::Connect(addr, tx) => {
                        let id = Uuid::new_v4();
                        let connection = Connection::new(tx, id);
                        self.connections.insert(addr, connection);
                    },
                    _ => {
                        println!("This command shouldn't be sent to the game");
                    }
                }
            }
        }

        // Poll each map
        //   Map Polls each item
        // The problem with map commands, is the shout commands. Those need
        // access to all other players.

        // Could access to other players be in sharedstate (main.rs) and then used inside Player? (server.rs)

        Ok(Async::NotReady)
    }

    pub fn create_mappings() -> HashMap<String, i16> {
        let mut m: HashMap<String,i16> = HashMap::new();
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
                    m.insert(img.file_stem().unwrap().to_str().unwrap().to_string(), count);
                    count = count + 1;
                    println!("{} {}", img.display(), count);
                },
                _ => {},
            }
        }
        return m;
    }
}

pub struct Connection {
    id: Uuid,
    tx: Tx,
}

impl Connection {
    fn new(tx: Tx, id: Uuid) -> Self {
        Connection {
            tx: tx,
            id: id
        }
    }
}

trait Character {
    fn build_auto_queue(map: Map);
    fn poll_execute() -> Poll<(), Error>;
}

pub struct Pc {
    id: Uuid,
    name: String,
    input_queue: Vec<Msg>,
    action_queue: Vec<Msg>,
    camera: Camera,
}

impl Pc {
    pub fn new (id: Uuid, name: &str, camera: Camera) -> Self {
        Pc {
            id: id,
            camera: camera,
            name: name.to_string(),
            input_queue: Vec::new(),
            action_queue: Vec::new(),
        }
    }

    pub fn add_command(&self, command: &str) {
        println!("{}", command);
    }
}

impl Character for Pc {

    fn build_auto_queue(map: Map) {

    }

    fn poll_execute() -> Poll<(), Error> {
        Ok(Async::NotReady)
    }
}

pub struct Npc {

}

impl Npc {
    fn new() -> Self {
        Npc {}
    }
}

impl Character for Npc {
    fn build_auto_queue(map: Map) {

    }

    fn poll_execute() -> Poll<(), Error> {
        Ok(Async::NotReady)
    }
}

pub struct Camera {
    width: u32,
    height: u32,
}

impl Camera {
    fn new(width: &u32, height: &u32) -> Self {
        Camera{
            width: width.clone(),
            height: height.clone(),
        }
    }

    fn poll_capture_snapshot() {

    }
}

pub struct Map {

}

impl Map {
    fn new() -> Self {
        Map{}
    }
}