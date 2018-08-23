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


use bytes::BytesMut;
use conn::{Msg, Login};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::collections::HashMap;
use std;
use std::str;
use tokio::prelude::*;
use tokio::net::TcpStream;
use tokio::io;
use time;
use std::fs::File;

//use game::gamemap::MapScreen;

/// This game module handles the API for the wyvern client. It only implements part of the API, as
/// documented on the wyvern website.
///
/// It also covers things like writing larger than 8 bit numbers to a u8 vector. (with varying
/// endiannes, as described in the API)
pub trait Api {
    fn write_header(vec: &mut Vec<u8>, code:u8, lenth:i32);
    fn write_string(vec: &mut Vec<u8>, string: &str);
    fn write_i16(vec: &mut Vec<u8>, short: i16);
    fn write_i16_reversed(vec: &mut Vec<u8>, short: i16);
    fn write_i32(vec: &mut Vec<u8>, val: i32);
    fn write_i32_reversed(vec: &mut Vec<u8>, val: i32);
    fn write_i64(vec: &mut Vec<u8>, val: i64);
    fn write_timestamp(vec: &mut Vec<u8>);
    fn write_conn_result(&mut self, result: u8, message: &str);
    fn write_quit(&mut self);
    fn write_tile_mappings(&mut self, mappings: HashMap<String, i16>);
    fn write_tile(&mut self, tile: i16, path: &str);
    fn write_image(&mut self, image: &str);
    fn write_text_out(&mut self, style: u8, message: &str);
    //fn write_zipped_screen(&mut self, screen: MapScreen);
    fn write_stat_name(&mut self, name: &str);
    fn write_stat(&mut self, stat: Stat, gold: i32);
    fn write_stat_level(&mut self,level: u8, xp: i32);
    fn write_stat_all(&mut self, hp: i32, mhp: i32, sp: i32, msp: i32, level: i32, xp: i32, nxp:
                      i32, food: i32, mfood: i32);
    fn write_ground_add(&mut self, name: &str, commands: &str, tile: i16, index: i16, offsets: i16);
    fn write_inv_add(&mut self, name: &str, commands: &str, tile: i16, index: i16, offsets: i16);
    fn zip_data(data: Vec<u8>) -> Vec<u8>;
}

pub enum Stat {
    Gold,
    Xp,
    Hp,
    Sp,
}

pub struct Codec {
    pub socket: TcpStream,
    logged_in: bool,
    input: Vec<Msg>,
    input_buffer: BytesMut,
    output_buffer: BytesMut,
}

impl Codec {
    pub fn new(socket: TcpStream) -> Self {
        let input = vec![];
        let input_buffer = BytesMut::new();
        let output_buffer = BytesMut::new();
        Codec {
            socket,
            logged_in: false,
            input,
            input_buffer,
            output_buffer,
        }
    }

    fn poll_login(&mut self) -> Poll<Option<Login>, io::Error> {
        //Do Login stuff
        self.input_buffer.reserve(1024);
        let number = try_ready!(self.socket.read_buf(&mut self.input_buffer));

        // This indicates a closed conn.
        if number == 0 {
            return Ok(Async::Ready(None));
        }

        let buf = &mut self.input_buffer;
        if number > 17 {
            let first_value = buf.split_to(4)[..].iter().fold(0i32, |sum, x| sum  << 8 | *x as i32);
            if first_value == 1 {
                let height = buf.split_to(2)[..].iter().fold(0u32, |sum, x| sum << 8 | *x as u32);
                let width = buf.split_to(2)[..].iter().fold(0u32, |sum, x| sum << 8 | *x as u32);
                let name_len = buf.split_to(2)[..].iter().fold(0usize, |sum, x| sum << 8 | *x as usize);
                let name_utf = buf.split_to(name_len);
                let name = std::str::from_utf8(&name_utf[..]).unwrap();
                let pw_len= buf.split_to(2)[..].iter().fold(0usize, |sum, x| sum << 8 | *x as usize);
                let pw_utf = buf.split_to(pw_len);
                let pw = std::str::from_utf8(&pw_utf[..]).unwrap();
                let version_len  = buf.split_to(2)[..].iter().fold(0usize, |sum, x| sum << 8 | *x as usize);
                let version_utf = buf.split_to(version_len);
                let version = std::str::from_utf8(&version_utf[..]).unwrap();
                return Ok(Async::Ready(Some(Login::new(height, width, name, pw, version))));
            }
        }
        Ok(Async::NotReady)
    }

    fn fill_read_buffer(&mut self) -> Poll<(), io::Error> {
        let addr = self.socket.peer_addr()?;
        self.input_buffer.reserve(4096);
        let number = try_ready!(self.socket.read_buf(&mut self.input_buffer));
        let buf = &mut self.input_buffer;

        if number == 0 {
            return Ok(Async::Ready(()));
        }

        loop {
            if buf.len() >= 3 {
                //Must be over 3 in length.
                let length = buf.split_to(2).iter().fold(0usize,| total, x | total  << 8 | *x as usize);
                if buf.len() >= 2 + length {
                    let command_utf =  buf.split_to(length);
                    let command =  str::from_utf8(&command_utf[..]).unwrap();
                    self.input.push(Msg::Command(addr, command.to_owned()));
                } else {
                    return Ok(Async::NotReady);
                }
            } else {
                return Ok(Async::NotReady);
            }
        }
    }


    pub fn flush_write_buffer(&mut self) -> Poll<(), io::Error> {
        while !self.output_buffer.is_empty() {
            let n = try_ready!(self.socket.poll_write(&self.output_buffer));

            self.output_buffer.split_to(n);
        }

        Ok(Async::Ready(()))
    }

    pub fn buffer(&mut self, message: Msg) {
        match message {
            Msg::TextOutput(result, message) => {
                // Write message
                self.write_text_out(result, &message);
            },
            //Msg::Screen(screen) => {
            //    //Write screen
            //    self.write_zipped_screen(screen);
            //},
            Msg::Shout(msg) => {
                self.write_text_out(4,&msg);
            },
            Msg::LoginResult(result, message) => {
                self.write_conn_result(result, &message);
            },
            Msg::Name(name) => {
                self.write_stat_name(&name);
            },
            Msg::Quit => {
                self.write_quit();
            },
            Msg::Image(img) => {
                self.write_image(&img);
            },
            Msg::Hp(amount) => {
                self.write_stat(Stat::Hp, amount);
            },
            Msg::Mana(amount) => {
                self.write_stat(Stat::Sp, amount);
            },
            Msg::Gold(amount) => {
                self.write_stat(Stat::Gold, amount);
            },
            Msg::Xp(amount) => {
                self.write_stat(Stat::Xp, amount);
            },
            Msg::Tile(tile, path) => {
                self.write_tile(tile, &path);
            },
            Msg::TileMapping(map) => {
                self.write_tile_mappings(map);
            },
            _ => {
                panic!("Oh no!");
            }
        }
    }
}


impl Stream for Codec {
    type Item = Msg;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if !self.logged_in {
            let addr = self.socket.peer_addr()?;
            let login = self.poll_login()?;

            match login {
                Async::Ready(Some(login)) => self.input.push(Msg::Login(addr, login)),
                Async::Ready(None) => {
                    return Ok(Async::Ready(None));
                },
                _ => {},
            }
            self.logged_in = true;
        }

        let sock_closed = self.fill_read_buffer()?.is_ready();

        if self.input.len() > 0 {
            let msg = self.input.remove(0);
            return Ok(Async::Ready(Some(msg)));
        }

        if sock_closed {
            Ok(Async::Ready(None))
        } else {
            Ok(Async::NotReady)
        }
    }
}

///Just implements the Api trait. This implementation is created from reading the web page that was
///posted with all this information.
impl Api for Codec {
    fn write_header(vec: &mut Vec<u8>, code:u8, length:i32) {
        vec.push(code);
        let high = length & 0xff0000;
        vec.push((high >> 16) as u8);
        let mid = length & 0xff00;
        vec.push((mid >> 8)  as u8);
        let low = length & 0xff;
        vec.push(low as u8);
    }

    fn write_string(vec: &mut Vec<u8>, string: &str) {
        let high= string.len() & 0xff00;
        vec.push((high >> 8) as u8);
        let low = string.len() & 0xff;
        vec.push(low as u8);
        vec.extend(string.as_bytes());
    }

    fn write_i16(vec: &mut Vec<u8>, short: i16) {
        let high= short as u16 & 0xff00;
        vec.push((high >> 8) as u8);
        let low = short & 0xff;
        vec.push(low as u8);
    }

    fn write_i16_reversed(vec: &mut Vec<u8>, short: i16) {
        let low = short & 0xff;
        vec.push(low as u8);
        let high= short as u16 & 0xff00;
        vec.push((high >> 8) as u8);
    }

    fn write_i32(vec: &mut Vec<u8>, val: i32) {
        let high = val as u32  & 0xff000000;
        vec.push((high >> 24) as u8);
        let mid_high = val & 0xff0000;
        vec.push((mid_high >> 16) as u8);
        let mid_low = val & 0xff00;
        vec.push((mid_low >> 8) as u8);
        let low = val & 0xff;
        vec.push(low as u8);
    }

    fn write_i32_reversed(vec: &mut Vec<u8>, val: i32) {
        let low = val & 0xff;
        vec.push(low as u8);
        let mid_low = val & 0xff00;
        vec.push((mid_low >> 8) as u8);
        let mid_high = val & 0xff0000;
        vec.push((mid_high >> 16) as u8);
        let high = val as u32  & 0xff000000;
        vec.push((high >> 24) as u8);
    }

    fn write_i64(vec: &mut Vec<u8>, val: i64) {
        let very_high = val as u64 & 0xff00000000000000;
        vec.push((very_high >> 56) as u8);
        let very_mid_high = val & 0xff000000000000;
        vec.push((very_mid_high >> 48) as u8);
        let very_mid_low = val &    0xff0000000000;
        vec.push((very_mid_low >> 40) as u8);
        let very_low = val &          0xff00000000;
        vec.push((very_low >> 32 )as u8);
        let high = val                & 0xff000000;
        vec.push((high >> 24) as u8);
        let mid_high = val & 0xff0000;
        vec.push((mid_high >> 16) as u8);
        let mid_low = val & 0xff00;
        vec.push((mid_low >> 8) as u8);
        let low = val & 0xff;
        vec.push(low as u8);
    }

    fn write_timestamp(vec: &mut Vec<u8>) {
        //println!("Timestamp");
        let current = time::get_time();
        let mut milli = current.sec * 1000;
        milli = milli + (current.nsec as i64 / 1000000);
        Codec::write_i64(vec, milli);
    }

    fn write_conn_result(&mut self, result: u8, message: &str) {
        let mut conn: Vec<u8> = vec![];
        Codec::write_header(&mut conn, 2, 5);
        //major version
        Codec::write_i16(&mut conn, 1);
        //minor version
        Codec::write_i16(&mut conn, 1);
        conn.push(result);
        if result == 4 {
            conn.extend(message.as_bytes());
        }
        //connection result indication
        //result
        //So, ints, and the utf size are done in diferent directions.
        //conn.push(0);
        //conn.push(6);
        //conn.extend("Failed".as_bytes());
        self.output_buffer.extend_from_slice(&conn[..]);
    }

    fn write_quit(&mut self) {
        //println!("Quit");
        //quit indication
        let mut q: Vec<u8> = vec![];
        Codec::write_header(&mut q, 13, 0);
        self.output_buffer.extend_from_slice(&q[..]);
    }

    fn write_tile_mappings(&mut self, mappings: HashMap<String, i16>) {
        //println!("Mappings");
        //Read map files. Write out entire output.
        let mut uncompressed: Vec<u8> = vec![];
        for (k, v) in mappings.iter() {
           Codec::write_i16(&mut uncompressed, v.clone());
           Codec::write_string(&mut uncompressed, k.trim());
        }
        let u_len = uncompressed.len();
        let mut compressed = Codec::zip_data(uncompressed);
        let z_len = compressed.len();
        let total = z_len + 8;
        let mut tile: Vec<u8> = vec![];
        Codec::write_header(&mut tile, 8, total as i32);
        //zipped len
        Codec::write_i32(&mut tile, z_len as i32);
        //ulen
        Codec::write_i32(&mut tile, u_len as i32);
        //zipped contents
        tile.append(&mut compressed);
        self.output_buffer.extend_from_slice(&tile[..]);
    }

    fn write_image(&mut self, image: &str) {
        let mut data:  Vec<u8> = vec![];
        match File::open(format!("images/{}.gif",image)) {
            Ok(mut file) => {
                let mut buf = vec![];
                match file.read_to_end(&mut buf) {
                    Ok(len) => {
                        Codec::write_header(&mut data, 17, (image.len() + 4 + len + 8) as i32);
                        Codec::write_string(&mut data, image);
                        Codec::write_i32(&mut data, len as i32);
                        data.append(&mut buf);
                        Codec::write_timestamp(&mut data);
                        self.output_buffer.extend_from_slice(&data[..]);
                        println!("Wrote tile");
                    },
                    _ => {},
                }

            },
            _ => {},
        }
    }

    fn write_tile(&mut self, tile: i16, path: &str) {
        //println!("Write tile");
        let mut buf: Vec<u8> = vec![];
        Codec::write_header(&mut buf, 16, 12 + path.len() as i32);
        Codec::write_i16(&mut buf, tile);
        Codec::write_string(&mut buf, path);
        Codec::write_timestamp(&mut buf);
        self.output_buffer.extend_from_slice(&buf[..]);
    }

    fn write_text_out(&mut self, style: u8, message: &str) {
        //println!("Text Out");
        let mut m: Vec<u8> = vec![];
        //text out byte
        Codec::write_header(&mut m, 11, (message.len()+3) as i32);
        m.push(style);
        Codec::write_string(&mut m, message);
        self.output_buffer.extend_from_slice(&m[..]);
    }

    //fn write_zipped_screen(&mut self, screen: MapScreen) {
    //    //Convert MapScreen to zipped screen.
    //    let mut uncompressed: Vec<u8> = vec![];
    //    for terrain in screen.terrain.iter() {
    //        let priority = terrain.get_priority();
    //        terrain.mapping.clone() as i32 | priority;
    //        Codec::write_i32_reversed(&mut uncompressed, t as i32);
    //    }
    //    //println!("Zipped screen");
    //    for object in screen.objects.iter() {
    //        //println!("Object! {}", object.tile);
    //        uncompressed.push(object.y);
    //        uncompressed.push(object.x);
    //        Codec::write_i16_reversed(&mut uncompressed, terrain.mapping.clone() as i16);
    //
    //    }
    //    //println!("Zipped screen");
    //    let u_len = uncompressed.len();
    //    let mut compressed = Codec::zip_data(uncompressed);
    //    let z_len = compressed.len();
    //    let total = 12 + z_len;
    //    let mut s : Vec<u8> = vec![];
    //    Codec::write_header(&mut s, 24, total as i32);
    //    //width
    //    Codec::write_i16(&mut s,screen.width);
    //    //height
    //    Codec::write_i16(&mut s,screen.height);
    //    //zipped len
    //    Codec::write_i32(&mut s,z_len as i32);
    //    //ulen
    //    Codec::write_i32(&mut s,u_len as i32);
    //    //zipped contents
    //    s.append(&mut compressed);
    //    self.output_buffer.extend_from_slice(&s[..]);
    //}

    fn write_stat_name(&mut self, name: &str) {
        //println!("Name");
        let mut n = vec![];
        Codec::write_header(&mut n, 106, 2 + name.len() as i32);
        Codec::write_string(&mut n, name);
        self.output_buffer.extend_from_slice(&n[..]);
    }


    fn write_stat(&mut self, stat: Stat, amount: i32) {
        //println!("Write gold");
        let mut n = vec![];
        let rpc_code = match stat {
            Stat::Hp => 100,
            Stat::Sp => 101,
            Stat::Xp => 102,
            Stat::Gold => 104,
        };
        Codec::write_header(&mut n, rpc_code, 4);
        Codec::write_i32(&mut n, amount);
        self.output_buffer.extend_from_slice(&n[..]);
    }

    fn write_stat_level(&mut self, level: u8, xp: i32) {
        //println!("Write level");
        let mut n = vec![];
        Codec::write_header(&mut n, 105, 5);
        n.push(level);
        Codec::write_i32(&mut n, xp);
        self.output_buffer.extend_from_slice(&n[..]);
    }

    fn write_stat_all(&mut self, hp: i32, mhp: i32, sp: i32, msp: i32, level: i32, xp: i32, nxp:
                      i32, food: i32, mfood: i32) {
        //println!("WRITE STATS");
        let mut n = vec![];
        Codec::write_header(&mut n, 120, 36 );
        Codec::write_i32(&mut n, hp);
        Codec::write_i32(&mut n, mhp);
        Codec::write_i32(&mut n, sp);
        Codec::write_i32(&mut n, msp);
        Codec::write_i32(&mut n, level);
        Codec::write_i32(&mut n, xp);
        Codec::write_i32(&mut n, nxp);
        Codec::write_i32(&mut n, food);
        Codec::write_i32(&mut n, mfood);
        self.output_buffer.extend_from_slice(&n[..]);
    }

    fn write_ground_add(&mut self, name: &str, commands: &str, tile: i16, index: i16, offsets: i16) {
        //println!("Write ground add {}", name);
        let mut n = vec![];
        Codec::write_header(&mut n, 80, (10 + name.len() + commands.len()) as i32);
        Codec::write_string(&mut n, name);
        Codec::write_string(&mut n, commands);
        Codec::write_i16(&mut n, tile);
        Codec::write_i16(&mut n, index);
        Codec::write_i16(&mut n, offsets);
        self.output_buffer.extend_from_slice(&n[..]);
    }

    fn write_inv_add(&mut self, name: &str, commands: &str, tile: i16, index: i16, offsets: i16) {
        //println!("Write inv add {}", name);
        let mut n = vec![];
        Codec::write_header(&mut n, 70, (10 + name.len() + commands.len()) as i32);
        Codec::write_string(&mut n, name);
        Codec::write_string(&mut n, commands);
        Codec::write_i16(&mut n, tile);
        Codec::write_i16(&mut n, index);
        Codec::write_i16(&mut n, offsets);
        self.output_buffer.extend_from_slice(&n[..]);
    }

    fn zip_data(data: Vec<u8> ) -> Vec<u8> {
        //println!("ZIP");
        let d = data.len();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::Default);
        let mut written = 0;
        while written < d {
            written = written + encoder.write(&data[written..]).unwrap();
        }
        encoder.finish().unwrap()
    }
}
