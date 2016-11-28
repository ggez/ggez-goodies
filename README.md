# ggez-goodies

[![Build Status](https://travis-ci.org/ggez/ggez-goodies.svg?branch=master)](https://travis-ci.org/ggez/ggez-goodies)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ggez/ggez/blob/master/LICENSE)


Useful modules for the ggez Rust game framework.

Each module is meant to be pretty much self-contained without relying on the others.
Or if it does rely on others, it will be in a strictly incremental fashion.

# Modules that exist

* Resource loader/cache (though it's a little janky)

# Modules to create

* Scene stack
* Particle system
* Input indirection layer and state tracking
* Sprites with ordering, animation, atlasing, tile mapping, 3x3 sprites... (look at
https://docs.rs/piston2d-sprite/0.28.0/sprite/index.html)
* GUI (conrod? imgui?)
* Timers?
* In-game debugger...

# Useful goodies made by other people

* specs for entity-component system (alternatives: ecs or recs or rustic-ecs crates)
* cgmath or vecmath for math operations?
* physics/collision???  ncollide.
* https://github.com/rsaarelm/calx
* https://github.com/Gekkio/imgui-rs

# Random thoughts

* Multithreading the particle system would be pretty cool!  split_at_mut seems the way.  (Nope,
this is harder than it looks and doesn't work as well as you hope at least with Rayon.  I've
been told to look into scoped threading crates for it.)

