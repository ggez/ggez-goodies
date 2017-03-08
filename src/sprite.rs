//! Sprites!
//! We want atlasing, flipbook animations, layering, tilemaps...

use std::cmp::Ordering;
use std::collections::BTreeMap;

use ggez;
use ggez::graphics;
use ggez::graphics::{Rect, Point, Drawable};


/// An object that contains metadata on an image atlas.
/// Does it contain the image itself or not?  For now, yes.
pub struct Atlas {
    source: graphics::Image,
    /// The number of sub-images across
    width: u32,
    /// The number of sub-images high
    height: u32,

    /// Width in pixels
    tile_width: u32,
    /// Height in pixels
    tile_height: u32,
}

impl Atlas {
    fn new(source: graphics::Image, width: u32, height: u32) -> Atlas {
        let tile_width = 128 / width;
        let tile_height = 128 / height;
        Atlas {
            source: source,
            width: width,
            height: height,
            tile_width: tile_width,
            tile_height: tile_height,
        }
    }
    fn get_source(&self, index: u32) -> ggez::GameResult<Rect> {
        Ok(Rect::new(0.0, 0.0, self.tile_width as f32, self.tile_height as f32))
    }
}

pub struct Sprite<'a> {
    atlas: &'a Atlas,
    index: u32,
}

impl<'a> graphics::Drawable for Sprite<'a> {
    fn draw_ex(&self,
               context: &mut ggez::Context,
               param: graphics::DrawParam)
               -> ggez::GameResult<()> {
        Ok(())
    }
}


impl<'a> Sprite<'a> {
    fn draw(&mut self,
            context: &mut ggez::Context,
            location: graphics::Point)
            -> ggez::GameResult<()> {
        let source = self.atlas.get_source(self.index)?;
        let dest = Rect::new(location.x, location.y, source.w, source.h);
        // grr why does this not work with the mutable Drawable
        // self.atlas.source.draw(context, Some(source), Some(dest))
        Ok(())
    }
}

struct LayerIndex {
    layer: i32,
    id: usize,
}

impl LayerIndex {
    fn new(layer: i32, id: usize) -> Self {
        LayerIndex {
            layer: layer,
            id: id,
        }
    }
}


impl PartialEq for LayerIndex {
    // Two objects are the same if their ID is identical.
    // all ID's should be unique, so.
    fn eq(&self, other: &LayerIndex) -> bool {
        self.id == other.id
    }
}

impl Eq for LayerIndex {}

impl PartialOrd for LayerIndex {
    fn partial_cmp(&self, other: &LayerIndex) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LayerIndex {
    fn cmp(&self, other: &LayerIndex) -> Ordering {
        if self.layer == other.layer {
            self.id.cmp(&other.id)
        } else {
            self.layer.cmp(&other.layer)
        }
    }
}


/// A `LayerManager` is in charge of doing all sprite drawing.
/// It has a collection of Drawable objects and will draw them
/// in order of layer and a monotonic ID that it manages on its
// own.
pub struct LayerManager<T>
    where T: Drawable
{
    layers: BTreeMap<LayerIndex, T>,
    next_id: usize,
}

impl<T: Drawable> LayerManager<T> {
    pub fn new() -> Self {
        LayerManager {
            layers: BTreeMap::new(),
            next_id: 0,
        }
    }

    fn next_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }

    pub fn add(&mut self, layer: i32, item: T) {
        let id = self.next_id();
        let idx = LayerIndex::new(layer, id);
        self.layers.insert(idx, item);
    }
}

impl<T: Drawable> Drawable for LayerManager<T> {
    fn draw_ex(&self,
               context: &mut ggez::Context,
               param: graphics::DrawParam)
               -> ggez::GameResult<()> {
        for (_key, item) in self.layers.iter() {
            graphics::draw_ex(context, item, param)?;
        }
        Ok(())
    }
}
