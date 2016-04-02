# MOBA

This is a MOBA built to work with an existing game client. 

Check out the [wyvernrpg subreddit](https://reddit.com/r/wyvernrpg) for info about the original game. 

Check the side bar for links to the game client and such.

# Running

* In order to build you need rust installed. Get Rust [here](https://www.rust-lang.org/downloads.html)
* Clone this project.
* Navigate to the cloned directory
* execute `cargo run`

# Connecting as client

* Fill in one of the player arts as username. Examples: panther_male, mage, paladin, malirith_male.
* Put anything in as the password. We don't care about it.
* Fill in the domain name/ip where the server is hosted. I have it up on map.rizato.com if you wish to try it out.

# Comptability

## Server
Due to some compatibility issues in one of the dependent projects, Mio, the server cannot run on Windows. 

## Client
This works with the original Wyvern client. It is compatible with clients running on any platform. 

Fortunately, we don't have to reverse anything to get that behavior. The entire protocol is documented, along with permission to use the art assets, [here](http://web.archive.org/web/20101121021755/http://cabochon.com/wiz/clients) and [here](http://web.archive.org/web/20101121031823/http://cabochon.com/wiz/client_protocol)

Additionally, because the API from Wyvern is subject to copyright, we obtained permission from the copyright holder to publish that portion of the code.