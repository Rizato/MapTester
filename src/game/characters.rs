use tokio::prelude::*;
use uuid::Uuid;

use game::command::{Command, CommandParser, CommandError};
use game::camera::Camera;
use game::map::Map;
use game::Point;

pub trait Character {
    fn build_auto_queue(&mut self, map: &Map);
    fn get_next(&mut self) -> Option<Command>;
}

pub struct Pc {
    pub id: Uuid,
    pub name: String,
    movement_queue: Vec<Command>,
    shoot_queue: Vec<Command>,
    chat_queue: Vec<Command>,
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

    pub fn add_command(&self, command: &str) -> Result<(), CommandError> {
        let command = CommandParser::new(command);
        let parsed = command.parse()?;
        match parsed {
            Command::Shout(message) => {
                Ok(())
            }
            _ => {
                Err(CommandError::new("Not implemented"))
            }
        }
    }
}

impl Character for Pc {
    fn build_auto_queue(&mut self, map: &Map) {

    }

    fn get_next(&mut self) -> Option<Command> {
        None
    }
}

pub struct Npc {
    commands: Vec<Command>,
}

impl Npc {
    fn new() -> Self {
        Npc {
            commands: Vec::new(),
        }
    }
}

impl Character for Npc {
    fn build_auto_queue(&mut self, map: &Map) {

    }

    fn get_next(&mut self) -> Option<Command> {
        self.commands.pop()
    }
}
