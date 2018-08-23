use tokio::prelude::*;
use uuid::Uuid;

use game::camera::Camera;
use game::map::Map;

pub trait Character {
    fn build_auto_queue(&mut self, map: &Map);
    // fn get_next(&mut self) -> Command;
}

pub struct Pc {
    pub id: Uuid,
    pub name: String,
    movement_queue: Vec<String>,
    shoot_queue: Vec<String>,
    chat_queue: Vec<String>,
    camera: Camera,
}

impl Pc {
    pub fn new (id: Uuid, name: &str, camera: Camera) -> Self {
        Pc {
            id: id,
            camera: camera,
            name: name.to_string(),
            movement_queue: Vec::new(),
            chat_queue: Vec::new(),
            shoot_queue: Vec::new(),
        }
    }

    pub fn add_command(&self, command: &str) {
        println!("{}", command);
    }
}

impl Character for Pc {
    fn build_auto_queue(&mut self, map: &Map) {

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
    fn build_auto_queue(&mut self, map: &Map) {

    }
}