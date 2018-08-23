use game::map::Map;

pub struct Camera {
    width: u32,
    height: u32,
}

impl Camera {
    pub fn new(width: &u32, height: &u32) -> Self {
        Camera{
            width: width.clone(),
            height: height.clone(),
        }
    }

    pub fn poll_capture_snapshot(&self, x: &u32, y: &u32, width: &u32, height: &u32, map: &Map) {

    }
}

// #[derive(Clone)]
// pub struct ScreenObject {
//     pub tile: String,
//     pub x: u8,
//     pub y: u8,
// }

// impl ScreenObject {
//     ///Creates a new screen object with the given tile, x and y
//     fn new(tile: String, x: u8, y:u8) -> ScreenObject{
//         ScreenObject{
//             tile: MapScreen::convert_terrain(tile),
//             x: x,
//             y: y,
//         }
//     }
// }

// ///This struct just holds a tile
// #[derive(Clone)]
// pub struct ScreenTerrain {
//     pub tile: String,
// }

// impl ScreenTerrain {
//     ///Creates a new tile struct
//     fn new(tile: String) -> ScreenTerrain {
//         ScreenTerrain {
//             tile: MapScreen::convert_terrain(tile),
//         }
//     }

//     ///To get fancy borders the tile has to have a priority assigned. I have
//     ///hardcoded some values here.
//     pub fn get_priority(&self) -> u32 {
//         let priority;
//         if self.tile.contains("grass") {
//             priority = (2 as u32) << 17;
//         } else if self.tile.contains("shallow") {
//             priority = (1 as u32) << 17;
//         } else if self.tile.contains("water") {
//             priority = (3 as u32) << 17;
//         } else if self.tile.contains("trees")
//             || self.tile.contains("forest")
//             || self.tile.contains("wall") {
//             priority = (4 as u32) << 17;
//         } else if self.tile.contains("lava") {
//             priority = (10000 as u32) << 17;
//         } else {
//             priority = (0 as u32) << 17;
//         }
//         priority | (7 as u32) << 29
//     }
// }

// ///Holds a vector of terrain objects, and a series of objects. The terrain is just a vector,
// ///representing a 15x15 matrix. The screen objects on the other hand denote their x and y
// ///specifically, rather than by their position in the array.
// #[derive(Clone)]
// pub struct MapScreen {
//     pub width: i16,
//     pub height: i16,
//     //15x15 vector.
//     pub terrain: Vec<ScreenTerrain>,
//     //User art at 7,7
//     pub objects: Vec<ScreenObject>,
// }

// impl MapScreen {
//     ///generates a new MapScreen based on the map and a given x & y. This will grab the 15x15
//     ///matrix centered on the given x and y. Any spaces beyond the boundaries of the map is replaced
//     ///with "empty" tiles
//     pub fn new(map: &GameMap, x: u32, y: u32, size_x: u8, size_y: u8) -> MapScreen {
//         let startx: isize = x as isize -(size_x as isize /2 as isize + 1);
//         let starty: isize = y as isize -(size_y as isize /2 as isize + 1);
//         let mut ter = Vec::with_capacity((size_x+2) as usize *(size_y+2) as usize);
//         let mut obj = vec![];
//         //If coords are valid we will actually draw something
//         let empty = ScreenTerrain::new("terrain/empty".to_string());
//         //creates array of tiles
//         if map.width as u32 > x && map.height as u32 > y {
//             for i in 0..(size_x as isize+2) {
//                 for j in 0..(size_y as isize+2) {
//                     if startx+i >= 0 && startx+(i as isize) < (map.width as isize) && starty+(j as isize) >= 0 && starty+(j as isize) < (map.height as isize) {
//                         //get the tile from the map
//                         let index= ((starty +j) * (map.width as isize)+ (startx+i)) as usize;
//                         let ref tiles = map.tiles;
//                         //clone the map tile
//                         let tile = tiles[index as usize].clone();
//                         //Add the terrain from the tile
//                         ter.push(ScreenTerrain::new(tile.tile.clone()));
//                     } else {
//                         ter.push(empty.clone());
//                     }
//                 }
//             }
//         }
//         //Creates array of objects
//         let ref objects = map.objects;
//         for i in 0..objects.len() {
//             let ref object = objects[i];
//             let index = object.get_location();
//             let object_x = index % map.width as u32;
//             let object_y = index / map.width as u32;
//             if object_x as isize >= startx && object_x < map.width as u32 && object_y as isize >= starty
//                 && object_y < map.height as u32  && object.is_visible(map) {
//                     //Extra -1 is to account for the extra tile off screen.
//                 obj.push(ScreenObject::new(object.get_tile(), (object_x as isize - startx -1) as u8 , (object_y as isize - starty -1) as u8));
//             }
//         }
//         MapScreen {
//             width: size_x as i16 +2,
//             height: size_y as i16 +2,
//             terrain: ter,
//             objects: obj,
//         }
//     }

//     ///These are the terrain tiles I have found where the terrain path from xml
//     ///did not match the image path.
//     fn convert_terrain(tile: String) -> String {
//         if tile == "scenery/sign1" {
//             "indoor/sign1".to_string()
//         } else if tile == "scenery/caveE" {
//             "scenery/cave2".to_string()
//         } else if tile == "terrain/dark_earth" {
//             "terrain/mud".to_string()
//         } else {
//             tile
//         }
//     }
// }