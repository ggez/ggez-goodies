# ggez-goodies

Useful modules for the ggez Rust game framework.

Each module is meant to be pretty much self-contained without relying on the others.
Or if it does rely on others, it will be in a strictly incremental fashion.

# Modules that exist

Bupkis!  Hah!

# Modules to create

* Resource loader/cache
* Scene stack
* Particle system
* Input indirection layer and state tracking
* Sprites with ordering, animation, atlasing, tile mapping, 3x3 sprites... (look at
https://docs.rs/piston2d-sprite/0.28.0/sprite/index.html)
* GUI (conrod backend?)
* Timers?
* In-game debugger...

# Useful goodies made by other people

* specs for entity-component system (alternatives: ecs or recs or rustic-ecs crates)
* cgmath or vecmath for math operations?
* physics/collision???  ncollide.
* https://github.com/rsaarelm/calx
* https://github.com/Gekkio/imgui-rs
