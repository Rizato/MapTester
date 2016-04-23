# Map Tester

This project started as an attempt to build a moba for the Wyvern client. 
Shortly after I started the original game creator announced the return of the real game. 
As a result, this project transitioned to a map tester, so that I could spend some time and get good at 
map development. 

Check out the [wyvernrpg subreddit](https://reddit.com/r/wyvernrpg) for info about the original game. 

Check the side bar for links to the game client & map editor (Go to the archived website).

# Run the server

I have a demo version set up at map.rizato.com. The only maps available there are "main" or "cave."

If you want to test your own maps you will have to set up your own instance. 
It is pretty easy as long as you have a Mac or Linux box available (or you can use one of the many compute cloud engines.)

* First, install rust. Get Rust [here](https://www.rust-lang.org/downloads.html)
* Clone this project.
* Navigate to the cloned directory
* execute `cargo run`

# Connecting as client

* Pick a username
* Put anything in as the password. Or nothing. 
* Fill in the domain name/ip where the server is hosted. I have it up on map.rizato.com if you wish to try it out.

## Movement

Right now, the map tester only supports mouse movement. This is because it was originally designed to be a moba.

I plan to add some other movement options soon.

## Available Commands

* skin <image>: Change to any image in the players directory. i.e skin paladin **Do not enter the .S/.N/.E/.W**
* join <map>: Change to a different map in the **maps/** directory
* shout <message>: Send a message to all other users on the server. Can be up to 4 KB long. 

## Changing Maps

There are two ways to switch maps. 

You can build a map with teleporters. When a user walks on a teleporter they are moved to the map that the teleporter points to. 
The perk of using it this way is that the developer can choose where in the map the person starts. This way you can have a map with different
entry points from different maps.

Additionally, a user can type `join <map>` into the games command bar. This will put the user at the default entry point for the map. 

*You can use teleporters or the join command to enter sub directories of **maps/**, but you cannot use a relative path with '..'*

# Testing a Map

1. Build a map with the map editor, available from the Wyvern archived website. 
2. Save the map as a .map file (Not Jython)
3. Move the file into the **maps/** directory. Can be placed inside a sub directory.
4. Start/Restart the server

# Custom Images

To add custom images drop them into the **images/** directory & restart the server. 

After adding them, any maps can use the custom images (though you may have to edit the map file by hand with the new image path)

If you want custom character images, they must be placed into **images/players/**

# Known Issues & Resolutions

If you run into a new issue, or these tips did not solve it for you, add an issue to the project. 

That said, this map tester definitely does not handle all the features of either the client or the map editor. It handle the subset that I needed, that is drawing terrain/objects and adding teleporters.

## Tile Placeholder Showing
![missing_small.png](https://bitbucket.org/repo/a6rebR/images/865065102-missing_small.png)

**Problem:** The image file specified was not found.

When using only stock images this is typically due to the path specified in XML not being an exact match for the image file. 

e.g. <arch path="/some/path.gif"/> but image file is /some/cool/path.gif.

**Resolution:** Two options. Edit the map file and put the actual image path. Edit Item or the ScreenTerrain structs and hard-code a string replace. To find the actual name of the file, right click the tile in the map editor & select all properties. The important fields are archetype, the path in XML, and image. 

![properties.png](https://bitbucket.org/repo/a6rebR/images/3302314735-properties.png)

I should probably have some sort of separate file with all of the conversions, but for now it is just hard-coded.

## Bad Borders
![small_bad_border.png](https://bitbucket.org/repo/a6rebR/images/2731151655-small_bad_border.png)

**Problem:** The tiles show hard edges instead of the nice transitions.

This is caused when the priority has not been set for some tile.

**Resolution:** Add another else if to the block inside ScreenTerrain::get_priority. 

```
pub fn get_priority(&self) -> u32 {
        let priority;
        if self.tile.contains("grass") {
            priority = (2 as u32) << 17; 
        [...]
        } else if self.tile.contains("my_missing_file") {
            priority = (8 as u32) << 17;
        } else {
            priority = (0 as u32) << 17;
        }
        priority | (7 as u32) << 29
    }
```

## Doors Don't Show
![small_door.png](https://bitbucket.org/repo/a6rebR/images/571234984-small_door.png)

**Problem:** Doors show as missing tile

Doors were a bit more complicated to get working than walls, so they just don't work yet.

You can currently walk through doors because you can't open them.

**Resolution:** Wait for me to get around to fixing them.

# Compatibility

## Server
Due to some compatibility issues in one of the dependent projects, Mio, the server only runs on Mac and Linux.

## Client
This works with the original Wyvern client. It is compatible with clients running on any platform. 

# Dev Priorities

* Add wsad/numpad/arrow key movement
* Doors
* Code Clean Up (Need to be more idiomatic Rust & better about my int types)
* Better update loop. (i.e. passing the map itself in the update function)
* Space themed zone. (Will require a bunch of custom art & some custom code, but should be pretty cool)
* Speed up adding players to map when entry is blocked.

## Adding features

If you want to add some more features that the Wyvern client supports check out the [protocol documentation](http://web.archive.org/web/20101121021755/http://cabochon.com/wiz/clients) and [here](http://web.archive.org/web/20101121031823/http://cabochon.com/wiz/client_protocol)

Additionally, because the API from Wyvern is subject to copyright, we obtained permission from the copyright holder to publish that portion of the code.

#License 
```
  Copyright 2016 Robert Lathrop

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
