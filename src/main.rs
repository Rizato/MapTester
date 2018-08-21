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


pub mod conn;

extern crate tokio;
#[macro_use]
extern crate futures;
extern crate flate2;
extern crate time;
extern crate xml;
extern crate glob;
extern crate bytes;

use conn::{Tx};
use conn::api::Codec;
use conn::server::Player;
use futures::sync::mpsc;
use std::sync::{Mutex, Arc};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::prelude::*;

/// This is the source for a MOBA server that is compatible with a preexisting game client.

fn main() {
    //This section starts up a tcp socket listening on port 2222, per the client docs
    println!("starting");

    let (tx, _rx) = mpsc::unbounded();
    // I need to figure out how to create the game here before the process_socket.
    let game = Arc::new(Mutex::new(SharedState::new(tx)));

    let addr: SocketAddr = "0.0.0.0:2222".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();


    let server = listener.incoming().for_each(move |socket| {
        let codec = Codec::new(socket);

        let player = Player::new(game.clone(), codec)
            .map_err(|e| {
                println!("Player Error: {:?}", e);
            });
        tokio::spawn(player);
        Ok(())
    })
    .map_err(|e| {
        println!("Conn Error: {:?}", e);
    });

    tokio::run(server);
}

pub struct SharedState {
    game: Tx,
}

impl SharedState {
    pub fn new(tx: Tx ) -> Self {
        SharedState {
            game: tx,
        }
    }
}
