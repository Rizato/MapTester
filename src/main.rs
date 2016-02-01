/*
  Copyright 2015 Robert Lathrop

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.*/

pub mod game;
pub mod conn;

extern crate mio;
extern crate flate2;
extern crate time;

use conn::server::Server;

use mio::tcp::*;
use std::net::SocketAddr;
use std::sync::Arc;

use std::cell::RefCell;
use std::sync::RwLock;


fn main() {
    println!("starting");
    let addr: SocketAddr = "0.0.0.0:2222".parse().unwrap();
    println!("addr");
    let server = TcpListener::bind(&addr).unwrap();
    println!("server");
    let mut event_loop = mio::EventLoop::new().unwrap();
    println!("event_loop"); 
    event_loop.register(&server, SERVER).unwrap();
    println!("register");

    let mut moba = Server::new(server);
    let _ = event_loop.run(&mut moba).unwrap();
}
