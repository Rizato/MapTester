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

use game::gamemap::MapScreen;

pub trait Api {
    fn write_header(vec: &mut Vec<u8>, code:u8, lenth:i32);
    fn write_string(vec: &mut Vec<u8>, string: &str);
    fn write_i16(vec: &mut Vec<u8>, short: i16);
    fn write_i16_reversed(vec: &mut Vec<u8>, short: i16);
    fn write_i32(vec: &mut Vec<u8>, val: i32);
    fn write_i32_reversed(vec: &mut Vec<u8>, val: i32);
    fn write_i64(vec: &mut Vec<u8>, val: i64);
    fn write_timestamp(vec: &mut Vec<u8>);
	fn write_conn_result(&mut self, result: u8);
	fn write_quit(&mut self);
	fn write_tile_mappings(&mut self);
	fn write_tile(&mut self, tile: i16);
	fn write_text_out(&mut self, style: u8, message: &str);
	fn write_zipped_screen(&mut self,screen: MapScreen);
    fn write_stat_name(&mut self, name: &str);
    fn write_stat_gold(&mut self, gold: i32);
    fn write_stat_level(&mut self,level: u8, xp: i32);
    fn write_stat_all(&mut self, hp: i32, mhp: i32, sp: i32, msp: i32, level: i32, xp: i32, nxp:
                      i32, food: i32, mfood: i32);
	fn zip_data(data: Vec<u8>) -> Vec<u8>;
}
