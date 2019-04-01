//! An efficient way of drawing images composed out of tiles from a spritesheet,
//! such as oldschool RPG maps.
//!
//! Includes a loader for the `tiled` map editor format.


use ggez::graphics;

/// A collection of layers, all the same size.
pub struct Map {
    layers: Vec<Layer>,
    width: usize,
    height: usize,
}

/// A single layer in the map.
/// Each item is a source rect, or None
/// if there is nothing to be drawn for that location,
/// which makes life a lot faster when drawing layered maps.
pub struct Layer {
    rects: Vec<Option<graphics::Rect>>,
}