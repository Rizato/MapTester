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

pub mod api;
///This just declares a couple more modules
pub mod player;

use futures::sync::mpsc;
use game::camera::MapScreen;
use std::collections::HashMap;
use std::net::SocketAddr;

pub type Tx = mpsc::UnboundedSender<Msg>;
pub type Rx = mpsc::UnboundedReceiver<Msg>;

/// enum for passing messages between connection & game loop. These are handled in the notify
/// method of the mio Handler
pub enum Msg {
    Command(SocketAddr, String),
    Connect(SocketAddr, Tx),
    Timeout(SocketAddr),
    Login(SocketAddr, Login),
    Image(String),
    Name(String),
    Hp(i32),
    Mana(i32),
    Gold(i32),
    Xp(i32),
    LoginResult(u8, String),
    Quit,
    Screen(MapScreen),
    Shout(String),
    TextOutput(u8, String),
    Tile(i16, String),
    TileMapping(HashMap<String, i16>),
}

// TODO Login should be a future that hits a DB to pull login info
pub struct Login {
    pub height: u32,
    pub width: u32,
    pub name: String,
    pub password: String,
    pub version: String,
}

impl Login {
    pub fn new(height: u32, width: u32, name: &str, password: &str, version: &str) -> Self {
        Login {
            height,
            width,
            name: name.to_owned(),
            password: password.to_owned(),
            version: version.to_owned(),
        }
    }
}
