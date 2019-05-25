use game::characters::Pc;
use game::map::Map;
use game::Point;
use std::collections::HashMap;

pub struct Camera {
    width: u32,
    height: u32,
}

impl Camera {
    pub fn new(width: &u32, height: &u32) -> Self {
        Camera {
            width: width.clone(),
            height: height.clone(),
        }
    }

    pub fn capture_snapshot(&self, player: Pc, map: &Map) -> Option<MapScreen> {
        if let Some(ref point) = map.players.get(&player.id) {
            return Some(MapScreen::new(
                &map,
                Point::new(&point.x, &point.y),
                self.width,
                self.height,
            ));
        }
        None
    }
}

#[derive(Clone)]
pub struct ScreenObject {
    pub tile: String,
    pub x: u32,
    pub y: u32,
}

impl ScreenObject {
    ///Creates a new screen object with the given tile, x and y
    fn new(tile: String, x: u32, y: u32) -> ScreenObject {
        let conversions = MapScreen::conversion_map();
        ScreenObject {
            tile: MapScreen::convert_terrain(conversions, tile),
            x: x,
            y: y,
        }
    }
}

///This struct just holds a tile
#[derive(Clone)]
pub struct ScreenTerrain {
    pub tile: String,
}

impl ScreenTerrain {
    ///Creates a new tile struct
    fn new(tile: String) -> ScreenTerrain {
        let conversions = MapScreen::conversion_map();
        ScreenTerrain {
            tile: MapScreen::convert_terrain(conversions, tile),
        }
    }

    ///To get fancy borders the tile has to have a priority assigned. I have
    ///hardcoded some values here.
    pub fn get_priority(&self) -> u32 {
        let priority;
        if self.tile.contains("grass") {
            priority = (2) << 17;
        } else if self.tile.contains("shallow") {
            priority = (1) << 17;
        } else if self.tile.contains("water") {
            priority = (3) << 17;
        } else if self.tile.contains("trees")
            || self.tile.contains("forest")
            || self.tile.contains("wall")
        {
            priority = (4) << 17;
        } else if self.tile.contains("lava") {
            priority = (10000) << 17;
        } else {
            priority = (0) << 17;
        }
        priority | (7) << 29
    }
}

///Holds a vector of terrain objects, and a series of objects. The terrain is just a vector,
///representing a 15x15 matrix. The screen objects on the other hand denote their x and y
///specifically, rather than by their position in the array.
#[derive(Clone)]
pub struct MapScreen {
    pub width: u32,
    pub height: u32,
    pub conversions: HashMap<String, String>,
    //15x15 vector.
    pub terrain: Vec<ScreenTerrain>,
    //User art at 7,7
    pub objects: Vec<ScreenObject>,
}

impl MapScreen {
    ///generates a new MapScreen based on the map and a given x & y. This will grab the 15x15
    ///matrix centered on the given x and y. Any spaces beyond the boundaries of the map is replaced
    ///with "empty" tiles
    pub fn new(map: &Map, center: Point, size_x: u32, size_y: u32) -> MapScreen {
        let startx = center.x - (size_x / 2 + 1);
        let starty = center.y - (size_y / 2 + 1);
        let mut ter = Vec::with_capacity((size_x + 2) as usize * (size_y + 2) as usize);
        let mut obj = vec![];
        //If coords are valid we will actually draw something
        let empty = ScreenTerrain::new("terrain/empty".to_string());
        //creates array of tiles
        if map.width > center.x && map.height > center.y {
            for i in 0..(size_x + 2) {
                for j in 0..(size_y + 2) {
                    if startx + (i) < (map.width) && starty + (j) < (map.height) {
                        //get the tile from the map
                        let index = ((starty + j) * (map.width) + (startx + i)) as usize;
                        let ref tiles = map.tiles;
                        //clone the map tile
                        let tile = tiles[index as usize].clone();
                        //Add the terrain from the tile
                        ter.push(ScreenTerrain::new(tile.tile.clone()));
                    } else {
                        ter.push(empty.clone());
                    }
                }
            }
        }
        //Creates array of objects
        let ref objects = map.objects;
        for object in objects {
            let path = object.path.clone();
            let mut location = &object.location;
            if location.x >= startx
                && location.x < map.width
                && location.y >= starty
                && location.y < map.height
                && object
                    .attributes
                    .get("visible")
                    .unwrap_or(&"".to_string())
                    .to_string()
                    == "true".to_string()
            {
                //Extra -1 is to account for the extra tile off screen.
                obj.push(ScreenObject::new(
                    path.unwrap_or("".to_string()),
                    location.x - startx - 1,
                    location.y - starty - 1,
                ));
            }
        }
        MapScreen {
            width: size_x + 2,
            height: size_y + 2,
            conversions: MapScreen::conversion_map(),
            terrain: ter,
            objects: obj,
        }
    }

    fn conversion_map() -> HashMap<String, String> {
        let mut conversion = HashMap::new();
        conversion.insert("scenery/sign1".to_string(), "indoor/sign1".to_string());
        conversion.insert("scenery/caveE".to_string(), "scenery/cave2".to_string());
        conversion.insert("terrain/dark_earth".to_string(), "terrain/mud".to_string());

        return conversion;
    }

    ///These are the terrain tiles I have found where the terrain path from xml
    ///did not match the image path.
    fn convert_terrain(conversion_map: HashMap<String, String>, tile: String) -> String {
        return match conversion_map.get(&tile) {
            Some(converted) => converted.to_string(),
            None => tile,
        };
    }
}
