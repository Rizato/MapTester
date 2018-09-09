use std::collections::{HashMap, VecDeque};
use tokio::prelude::*;
use uuid::Uuid;

use game::command::{Command, CommandParser, CommandError};
use game::camera::Camera;
use game::map::Map;
use game::Point;

pub trait Character {
    fn build_auto_queue(&mut self, map: &Map);
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
            // Do pathfinding as movement queue
            Command::Shout(message) => {
                Ok(())
            },
            Command::MoveTarget(point) => {
                Ok(())
            },
            _ => {
                Err(CommandError::new("Not implemented"))
            }
        }
    }

    pub fn next_movement(&mut self, map: &Map, position: &Point) -> Option<Command> {
        None
    }
}

impl Character for Pc {
    fn build_auto_queue(&mut self, map: &Map) {

    }
}

// This delegate contains how to respond to all the different actions that can be taken
#[derive(Clone)]
pub struct NpcDelegate {
    pub when_activated: Option<fn(&Uuid) -> Command>,
}

impl NpcDelegate {
    fn new(when_activated: Option<fn(&Uuid) -> Command>) -> Self {
        NpcDelegate {
            when_activated: when_activated
        }
    }
}

pub struct Npc {
    name: Option<String>,
    path: Option<String>,
    pub location: Point,
    pub delegate: Option<NpcDelegate>,
    commands: VecDeque<Command>,
    pub attributes: HashMap<String, String>,

}

impl Npc {
    fn new(name: Option<String>, location: &Point, path: Option<String>, delegate: Option<NpcDelegate>) -> Self {
        Npc {
            path: path,
            commands: VecDeque::new(),
            location: location.clone(),
            delegate: delegate,
            attributes: HashMap::new(),
            name: name,
        }
    }

    fn get_next(&mut self) -> Option<Command> {
        // This pulls the last element added
        self.commands.pop_front()
    }
}

impl Character for Npc {
    fn build_auto_queue(&mut self, map: &Map) {

    }
}
