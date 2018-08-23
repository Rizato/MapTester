use tokio::prelude::*;
use uuid::Uuid;

use game::camera::Camera;
use game::map::Map;
use game::Command;

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

            }
        }
    }
}

impl Character for Pc {
    fn build_auto_queue(&mut self, map: &Map) {

    }

    fn get_next(&mut self) -> Option<Command> {

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
        let self.commands.pop()
    }
}

struct CommandParser {
    command: String,
}

impl CommandParser {
    fn new(command: &str) -> Self {
        CommandParser {
            command: command.to_string();
        }
    }

    fn parse(&self) -> Result<Command, CommandError> {
        let parts = self.command.split_whitespace());
        let first = parts.next();
        let rest = parts.fold(String::new(), |acc, x| write!("{} {}", acc, x));
        match first {
            "shout" => {
                parse_shout(parts)
            },
            "mouse" => {
                parse_mouse(parts)
            },
            "whisper" => {
                parse_whisper(parts)
            },
            _ => {
                CommandError::new("Invalid Command")
            },
        }
    }

    fn parse_whisper(command: Iterator) -> Command {
        let target = command.next();
        let rest = command.fold(String::new(), |acc, x| write!("{} {}", acc, x));
        Command::Whisper(target.to_string(), rest.to_string())
    }

    fn parse_shout(command: Iterator) -> Command {
        let rest = command.fold(String::new(), |acc, x| write!("{} {}", acc, x));
        Command::Shout(rest)
    }

    fn parse_mouse() -> Command {

    }
}

#[derive(Debug)]
struct CommandError {
    message: String,
}

impl CommandError {
    fn new(message: &str) -> Self {
        CommandError {
            message: message.to_string();
        }
    }
}

impl Error for CommandError {
    fn description(&self) -> String {
        &self.message
    }

    fn source(&self) -> Option<&Error> {
        None
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
