use std::error::Error;
use std::fmt;
use std::str::SplitWhitespace;

use tokio::prelude::*;
use uuid::Uuid;

use game::camera::Camera;
use game::map::Map;
use game::Command;
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

struct CommandParser {
    command: String,
}

impl CommandParser {
    fn new(command: &str) -> Self {
        CommandParser {
            command: command.to_string()
        }
    }

    fn parse(&self) -> Result<Command, CommandError> {
        let mut parts = self.command.split_whitespace();
        if let Some(first) = parts.next() {
            return match first {
                "shout" => {
                    CommandParser::parse_shout(parts)
                },
                "mouse" => {
                    CommandParser::parse_mouse(parts)
                },
                "whisper" => {
                    CommandParser::parse_whisper(parts)
                },
                _ => {
                    Err(CommandError::new(&format!("Command {} not recognized", first)))
                },
            };
        }
        Err(CommandError::new("Invalid Command"))
    }

    fn parse_whisper(mut command: SplitWhitespace) -> Result<Command, CommandError> {
        let target = command.next();
        match command.next() {
            Some(target) => {
                let rest = command.fold(String::new(), |acc, x| format!("{} {}", acc, x));
                Ok(Command::Whisper(target.to_string(), rest.to_string()))
            },
            None => {
                Err(CommandError::new("No whisper target"))
            },
        }
    }

    fn parse_shout(mut command: SplitWhitespace) -> Result<Command, CommandError> {
        let rest = command.fold(String::new(), |acc, x| format!("{} {}", acc, x));
        Ok(Command::Shout(rest))
    }

    fn parse_mouse(mut command: SplitWhitespace) -> Result<Command, CommandError> {
        if let Some(x) = command.next() {
            if let Some(y) = command.next() {
                let target = Point::new(&x.parse::<u32>().unwrap(), &y.parse::<u32>().unwrap());
                let bitfield = command.fold(String::new(), |acc, x| format!("{}{}", acc, x));
                if let Ok(button) = u8::from_str_radix(&bitfield, 2) {
                    // Could be improved using BitFlags crate
                    if button & 8 == 8 {
                        return Ok(Command::Shoot(target, 0, 0));
                    } else if button & 16 == 16 {
                        return Ok(Command::Move(target));
                    }
                }
            }
        }
        Err(CommandError::new("Could not parse mouse input."))
    }
}

#[derive(Debug)]
pub struct CommandError {
    message: String,
}

impl CommandError {
    fn new(message: &str) -> Self {
        CommandError {
            message: message.to_string()
        }
    }
}

impl Error for CommandError {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
