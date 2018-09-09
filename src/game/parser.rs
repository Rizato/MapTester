
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

use game::command::Command;
use game::map::Map;

struct Header {
    height: u32,
    width: u32,
    start_x: u32,
    start_y: u32,
    name: String,
    terrain: String,
    oob-terrain: String,
    pk: bool,
}

impl Header {
    // Find out real attributes here
    fn new(attributes: Any) -> Result<Self, String> {
        let mut width = 0u32;
        let mut height = 0u32;
        for attr in attributes {
            match attr.name.local_name {
                "width" => {
                    if let Ok(w) = attr.value.parse::<u32>() {
                        width = w;
                    }
                },
                "height" => {
                    if let Ok(h) = attr.value.parse::<u32>() {
                        height = h;
                    }
                },
                _ => {},
            }
        }

        if height == 0 or width == 0 {
            return Err(format!("Invalid height and width: {} {}", height, width));
        }

        Ok(Header {
            height: width,
            width: height,
            start_x: 0u32,
            start_y: 0u32,
            name: String::new(),
            terrain: String::new(),
            oob-terrain: String::new(),
            pk: false,
        })
    }
}

impl ChildParser for Header {
    fn parse_child_element(&self, name: String, attributes: attrs) {
        let mut attr_name = String::new();
        let mut value = String::new();
        let mut path = String::new();

        for attr in attributes {
            match attr.name.local_name {
                "name" => {
                    attr_name = attr.value;
                },
                "value" => {
                    value = attr.value;
                },
                "path" => {
                    path = attr.value;
                },
                _ => {},
            }
        }

        match name {
            "string" => {
                if attr_name == "name" {
                    self.name = value;
                }
            },
            "int" => {
                if attr_name == "startX" {
                    self.start_x = value;
                } else if attr_name == "startY" {
                    self.start_y = value;
                }
            },
            "arch" => {
                if attr_name == "oob-terrain" {
                    self.oob-terrain = path;
                } else if attr_name == "terrain" {
                    self.terrain = path;
                }
            },
            "boolean" => {
                if attr_name == "pk" {
                    self.pk = true;
                }
            },
            _ => {},
        }
    }

    fn modify_map(&self, map: &mut Map) {
        map.width = self.width;
        map.height = self.height;
        map.modifiers.insert("startX", self.start_x);
        map.modifiers.insert("startY", self.start_y);
        map.modifiers.insert("name", self.name);
        map.modifiers.insert("terrain", self.terrain);
        map.modifiers.insert("oob-terrain", self.oob-terrain);
        map.modifiers.insert("pk", self.pk);
        // Set all tiles at the start
        map.tiles. = vec![ MapTile::new(self.terrain); self.width * self.height ];
    }
}

struct Terrain {
    path: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Terrain {
    fn new(attributes: Any) -> Result<Self, String> {
        let mut loc: = "0 0 0 0".split_whitespace();
        let mut path = String::new();

        for attr in attributes {
            match attr.name.local_name {
                "loc" => {
                    loc = Some(attr.value.split_whitespace());
                },
                "path" => {
                    path = attr.value;
                },
            }
        }

        values = loc.map(|v| v.parse::<u32>().unwrap()).collect();
        if values.len() == 2 {
            for _ in 0..1 {
                values.push(1);
            }
        } else if values.len() != 4 {
            return Err("Invalid location value.");
        }

        Ok(Terrain {
            path: path,
            rect_values: values,
        })
    }
}

impl ChildParser for Terrain {
    fn parse_child_element(&mut self, name, attributes) {
        return;
    }

    fn modify_map(&self, &mut map: Map) {
        let rect_x = self.values[0];
        let rect_y = self.values[1];
        let rect_w = self.values[2];
        let rect_h = self.values[3];

        for x in rect_x..(rect_x+rect_w) {
            for y in rect_y..(rect_y+rect_h) {
                let index: usize = (y as usize * width as usize) as usize + x as usize;
                map.tiles[index].tile = tile.clone();
                map.tiles[index].blocked = false;
            }
        }
    }
}

struct TeleporterTarget {
    path: String,
    x: u32,
    y: u32,
    ask-map: bool,
}

#[derive(Clone)]
impl TeleporterTarget {
    fn new() -> Self {
        map: String::new(),
        x: 0u32,
        y: 0u32,
        ask-map: false,
    }
}

struct Teleporter {
    target: TeleporterTarget,
    location: Vec<u32>,
}

impl Teleporter {
    fn new() -> Self {
        let mut loc: = "0 0 0 0".split_whitespace();

        for attr in attributes {
            match attr.name.local_name {
                "loc" => {
                    loc = Some(attr.value.split_whitespace());
                }
            }
        }

        values = loc.map(|v| v.parse::<u32>().unwrap()).collect();
        if values.len() == 2 {
            for _ in 0..1 {
                values.push(1);
            }
        } else if values.len() != 4 {
            return Err("Invalid location value.");
        }
        Teleporter {
            target: TeleporterTarget::new(),
            location: values
        }
    }
}

impl ChildParser for Teleporter {
    fn parse_child_element(&mut self, name, attributes) {
        let mut attr_name = String::new();
        let mut value = String::new();

        for attr in attributes {
            match attr.name.local_name {
                "name" => {
                    attr_name = attr.value;
                },
                "value" => {
                    value = attr.value;
                }
                _ => {},
            }
        }

        match name {
            "string" => {
                if name == "map" {
                    self.target.map = value;
                }
            },
            "int" => {
                if attr_name == "x" {
                    self.target.x = value.parse::<u32>();
                } else if attr_name == "y" {
                    self.target.y = value.parse::<u32>();
                }
            },
            _ => {},
        }
    }

    fn modify_map(&self, &mut map: Map) {
        let rect_x = self.values[0];
        let rect_y = self.values[1];
        let rect_w = self.values[2];
        let rect_h = self.values[3];

        for x in rect_x..(rect_x+rect_w) {
            for y in rect_y..(rect_y+rect_h) {
                let target = self.target.clone();
                let teleporter = Npc::new(None, Point::new(x, y), None, |c| => {
                    Command::Teleport(c.clone(), target.map, target.ask_map, Point::new(target.x, target.y))
                }
                map.objects.push(teleporter);
            }
        }
    }
}

/// TODO: Must do Items, Roads & walls. Including the type of road.
struct Item {
    path: String,
    values: Vec<u32>,
}

impl Item {
    fn new(attributes: Any) -> Result<Self, String> {
        let mut loc: = "0 0 0 0".split_whitespace();
        let mut path = String::new();

        for attr in attributes {
            match attr.name.local_name {
                "loc" => {
                    loc = Some(attr.value.split_whitespace());
                },
                "path" => {
                    path = attr.value;
                },
            }
        }

        values = loc.map(|v| v.parse::<u32>().unwrap()).collect();
        if values.len() == 2 {
            for _ in 0..1 {
                values.push(1);
            }
        } else if values.len() != 4 {
            return Err("Invalid location value.");
        }

        Ok(Item {
            path: path,
            values: values,
        })
    }
}

impl ChildParser for Item {
    fn parse_child_element(&mut self, name, attributes) {
        return;
    }

    fn modify_map(&self, &mut map: Map) {
        let rect_x = self.values[0];
        let rect_y = self.values[1];
        let rect_w = self.values[2];
        let rect_h = self.values[3];

        for x in rect_x..(rect_x+rect_w) {
            for y in rect_y..(rect_y+rect_h) {
                let item = Npc::new(None, Point::new(x, y), Some(self.path.clone()), None);
                map.objects.push(item);
            }
        }
    }
}

trait ChildParser {
    fn parse_child_element(&mut self, name, attributes);
    fn modify_map(&self, &mut map: Map);
}

struct MapParser {
    element: Option<Box<ChildParser>>,
    nested: u32,
}

impl MapParser {
    fn new() -> Self {
        MapParser {
            elemen: None,
            nested: 0u32,
        }
    }

    ///Checks to see if the map exists
    fn maps_exist(path: &str) -> bool {
        if let Ok(_) File::open(path) {
            return true;
        }
        false
    }

    fn join_roads(&mut self) {
        //Telling roads & walls to draw based on surrounding tiles
        let mut roads = HashSet<Point>::new();
        let mut wall = HashSet<Point>::new();
        let len = self.objects.len();
        for i in 0..len {
            let ref mut npc = objects[i];
            if npc.path.contains("main_road") || npc.path.contains("roads") {
                roads.insert(npc.location.to_index(self.width))
            } else if npc.path.contains("walls") {
                walls.insert(npc.location.to_index(self.width))
            }
        }

        Parser::append_connected(roads);
        Parser::append_connected(walls);
    }

    fn append_connected(roads: &HashSet<Npc>)
        for mut npc in roads.iter_mut() {
            // Generate 4 check points
            let point = npc.location;
            if roads.contains(Point::new(point.x, point.y+1)) {
                npc.path.push('N');
            }

            if roads.contains(Point::new(point.x, point.y-1)) {
                npc.path.push('S');
            }

            if roads.contains(Point::new(point.x+1, point.y)) {
                npc.path.push('E');
            }

            if roads.contains(Point::new(point.x-1, point.y)) {
                npc.path.push('W');
            }
        }
    }

    fn parse(&mut self, path: &str) -> Result<Map, String> {
        if !MapParser::maps_exist(path) {
            return Err("Map Not Found".to_string());
        }

        // Create a new blank map
        let mut map = Map::new();

        if let Ok(map_file) = File::open(path) {
            let tile_mappings = Game::create_mappings();
                let buf = BufReader::new(map_file);
                let parser = EventReader::new(buf);
                for event in parser {
                    match event {
                        Ok(XmlEvent::StartElement {name, attributes, ..}) => {
                            if let Some(root) = self.element {
                                self.nested += 1;
                                root.parse(name.local_name, attributes);
                            } else {
                                self.element = match name.local_name {
                                    "header": {
                                        match Header::new(attributes) {
                                            Ok(h) => {
                                                Some(Box::new(h))
                                            },
                                            Err(e) => {
                                                Err(e)
                                            }
                                        }
                                    },
                                    "arch": {
                                        let mut path = String::new();
                                        for attr in attributes {
                                            if attr.name.local_name == "path" {
                                                path = attr.value;
                                            }
                                        }

                                        if path.contains("terrain") {
                                            match Terrain::new(attributes) {
                                                Ok(t) => {
                                                    Some(Box::new(t))
                                                },
                                                Err(e) => {
                                                    Err(e)
                                                }
                                            }

                                        } else if path == "special/teleporter" {
                                            Some(Box::new(Teleporter::new(attributes)))

                                        } else if path.contains("roads") {

                                        } else {
                                            Some(Box::new(Item::new(attributes)))
                                        }
                                    },
                                }
                            }
                        },
                        Ok(XmlEvent::EndElement {name, ..}) => {
                            if self.nested > 0 {
                                self.nested -= 1;
                            } else {
                                // Modify map based on element
                                if let Some(parser) = self.element {
                                    parser.modify_map(&map);
                                }
                                self.element = None;
                            }
                        },
                        _ => {},
                    }
        }

        self.join_roads()
        return Ok(map);
    }
    Err(format!("Failed to load {}", path))
}