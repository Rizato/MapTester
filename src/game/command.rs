/*
  Copyright 2018 Robert Lathrop

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License. */

use std::error::Error;
use std::fmt;
use std::str::SplitWhitespace;

use uuid::Uuid;

use game::characters::Character;
use game::Point;

pub enum Command {
    MoveTarget(Point),                   // Move to position
    MoveStep(Uuid, Point, Direction),    // Target, Move to position
    Shoot(Point, u32, u32),              // Dest(x,y), Hit, Damage
    Attack(Uuid, u32, u32),              // Target, Hit, Damage
    Shout(String),                       // Message
    Whisper(String, String),             // Name, Message
    Teleport(Uuid, String, bool, Point), // Target, Map, Ask map?, Location
    Respawn(Uuid),                       // Respawn character
}

pub enum Direction {
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
}

pub struct CommandParser {
    command: String,
}

impl CommandParser {
    pub fn new(command: &str) -> Self {
        CommandParser {
            command: command.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Command, CommandError> {
        let mut parts = self.command.split_whitespace();
        if let Some(first) = parts.next() {
            return match first {
                "shout" => CommandParser::parse_shout(parts),
                "mouse" => CommandParser::parse_mouse(parts),
                "whisper" => CommandParser::parse_whisper(parts),
                _ => Err(CommandError::new(&format!(
                    "Command {} not recognized",
                    first
                ))),
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
            }
            None => Err(CommandError::new("No whisper target")),
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
                        // TODO convert to raw map coordinate
                        return Ok(Command::MoveTarget(target));
                    }
                }
            }
            // let parts: Vec<&str> = command.split_whitespace().collect();
            // //Mouse click x,y
            // let mx = parts[1].parse::<i32>().unwrap();
            // let my = parts[2].parse::<i32>().unwrap();
            // //old x,y
            // let oy = p.get_location() / (self.width as u32);
            // let ox = p.get_location() % (self.width as u32);
            // //change in x,y. -6 cause user is always in middle of screen, no matter the click.
            // let dx = if ox as i32 + mx > 6 { ox + mx as u32 -6 } else {0};
            // let dy = if oy as i32 + my > 6 { oy + my as u32 -6 } else {0};
            // println!("Move to x{} y{}", dx, dy);
            // let end = dy * self.width as u32 + dx;
        }
        Err(CommandError::new("Could not parse mouse input."))
    }
}

#[derive(Debug)]
pub struct CommandError {
    message: String,
}

impl CommandError {
    pub fn new(message: &str) -> Self {
        CommandError {
            message: message.to_string(),
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
