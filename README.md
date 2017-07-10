# ggez-goodies

[![Build Status](https://travis-ci.org/ggez/ggez-goodies.svg?branch=master)](https://travis-ci.org/ggez/ggez-goodies)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ggez/ggez/blob/master/LICENSE)


Useful modules for the ggez Rust game framework.

Each module is meant to be pretty much self-contained without relying on the others.
Or if it does rely on others, it will be in a strictly incremental fashion.

CURRENTLY this is massively unstable and is basically changing fluidly as I discover what the API needs to be like.

# Moduels that are actually useful/useable:

* Input indirection layer and state tracking
* Scene manager
* Camera

# Other modules that exist

* Resource loader/cache (though it's a little janky)
* Particle system (incomplete)

# Modules to create

* Sprites with ordering, animation, atlasing, tile mapping, 3x3 sprites... (look at https://docs.rs/piston2d-sprite/0.28.0/sprite/index.html) (also look at XNA's sprite system, FNA's implementation of such)
* Font renderer/cacher that creates a bitmap font from a TTF.
* GUI (conrod? imgui?)
* Timers?  Coroutines?
* In-game debugger...

# Useful goodies made by other people

* specs for entity-component system (alternatives: ecs or recs or rustic-ecs crates)
* cgmath, nalgebra or vecmath for math operations
* physics/collision???  ncollide.
* https://github.com/rsaarelm/calx
* https://github.com/Gekkio/imgui-rs
* https://github.com/BourgondAries/TileNet
* There's a couple crates for loading Tiled maps, check 'em out