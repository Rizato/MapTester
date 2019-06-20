//   Copyright 2016 Robert Lathrop

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod conn;
pub mod game;

extern crate tokio;
#[macro_use]
extern crate futures;
extern crate bytes;
extern crate flate2;
extern crate glob;
extern crate time;
extern crate uuid;
extern crate xml;
#[macro_use]
extern crate log;

use conn::api::Codec;
use conn::player::Player;
use conn::Tx;
use futures::sync::mpsc;
use game::{Game, GameMessageParser};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::Interval;

/// This is the source for a MOBA server that is compatible with a preexisting game client.

fn main() -> Result<(), io::Error> {
    info!("starting");

    let (tx, rx) = mpsc::unbounded();
    let mut game = Arc::new(Mutex::new(Game::new(rx)));
    let gameloop = Interval::new(Instant::now(), Duration::from_millis(15))
        .map_err(move |_| {
            error!("Error in interval");
        })
        .for_each(move |_| {
            let game = game.clone();
            const COMMANDS_PER_TICK: u64 = 50;
            GameMessageParser::new(&game.clone(), COMMANDS_PER_TICK).and_then(move |_| {
                let mut game = game.lock();

                match game {
                    Ok(ref mut g) => g.tick(),
                    Err(e) => {
                        error!("error getting game lock");
                        Err(())
                    }
                }
            })
        });

    let shared = Arc::new(Mutex::new(SharedState::new(tx)));

    let addr: SocketAddr = "0.0.0.0:2222".parse().unwrap();
    let listener = TcpListener::bind(&addr)?;
    // Standup the server
    let server = listener
        .incoming()
        .for_each(move |socket| {
            let codec = Codec::new(socket);

            let player = Player::new(shared.clone(), codec).map_err(|e| {
                error!("Player Error: {:?}", e);
            });
            tokio::spawn(player);
            Ok(())
        })
        .map_err(|e| {
            error!("Conn Error: {:?}", e);
        });

    // Start the gameloop in one backend, the main thing in a blocking runtime
    // The docs define rt.block_on, but it didn't exist when I tried to compile
    let mut rt = Runtime::new().unwrap();
    rt.spawn(gameloop);
    tokio::run(server);
    Ok(())
}

pub struct SharedState {
    game: Tx,
}

impl SharedState {
    pub fn new(tx: Tx) -> Self {
        SharedState { game: tx }
    }
}
