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

use conn::api::Codec;
use conn::{Msg, Rx};
use futures::sync::mpsc;
use futures::Future;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io;
use tokio::prelude::*;
use SharedState;

/// This module defines the player struct.
///
/// Player is a future that polls the codec channel and passes the data
/// to the game channel

pub struct Player {
    game: Arc<Mutex<SharedState>>,
    addr: SocketAddr,
    codec: Codec,
    rx: Rx,
}

impl Player {
    pub fn new(game: Arc<Mutex<SharedState>>, codec: Codec) -> Self {
        let (tx, rx) = mpsc::unbounded();
        let addr = codec.socket.peer_addr().unwrap();
        game.lock()
            .unwrap()
            .game
            .unbounded_send(Msg::Connect(addr, tx))
            .unwrap();

        Player {
            game,
            addr,
            codec,
            rx,
        }
    }
}

impl Future for Player {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        const MESSAGES_PER_TICK: usize = 10;
        // read rx
        // buffer messages
        for _x in 0..MESSAGES_PER_TICK {
            match self.rx.poll().unwrap() {
                Async::Ready(Some(msg)) => {
                    self.codec.buffer(msg);
                }
                _ => {
                    break;
                }
            }
        }

        // poll flush - We don't try_ready here because we don't care if it fails. We will
        // find out that the socket is closed below, or we will end up doing another rotation if it
        // isn't.
        self.codec.flush_write_buffer()?;

        // So msg, is an option. If it is none, we return Async::Ready. If poll returns
        // NotReady, we skip this and hit NotReady.
        while let Async::Ready(msg) = self.codec.poll()? {
            if let Some(m) = msg {
                self.game.lock().unwrap().game.unbounded_send(m).unwrap();
            } else {
                // remove client from game

                self.game
                    .lock()
                    .unwrap()
                    .game
                    .unbounded_send(Msg::Timeout(self.addr.clone()))
                    .unwrap();
                return Ok(Async::Ready(()));
            }
        }
        // In order to get here, we have to be still connected, or we would have met the EOF above.
        Ok(Async::NotReady)
    }
}
