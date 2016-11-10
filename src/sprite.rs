//! Sprites!
//! We want atlasing, flipbook animations, layering, tilemaps...

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
        Ok(Rect::new(0, 0, self.tile_width, self.tile_height))
    }
}

pub struct Sprite<'a> {
    atlas: &'a Atlas,
    index: u32,
}

impl<'a> graphics::Drawable for Sprite<'a> {
    fn draw_ex(&self,
               context: &mut ggez::Context,
               src: Option<graphics::Rect>,
               dst: Option<graphics::Rect>,
               angle: f64,
               center: Option<graphics::Point>,
               flip_horizontal: bool,
               flip_vertical: bool)
               -> ggez::GameResult<()> {
        Ok(())
    }
}


impl<'a> Sprite<'a> {
    fn draw(&self, context: &mut ggez::Context, location: graphics::Point) -> ggez::GameResult<()> {
        let source = try!(self.atlas.get_source(self.index));
        let dest = Rect::new(location.x(), location.y(), source.width(), source.height());
        self.atlas.source.draw(context, Some(source), Some(dest))
    }
}



/// A `SpriteManager` is in charge of doing all sprite drawing.
/// It manages `Atlas`es, `Sprite`s, and so on.
/// When you tell it to draw, it will draw all sprites,
/// doing layering and such.
struct SpriteManager<'a> {
    atlas: Atlas,
    sprites: Vec<Vec<Sprite<'a>>>,
}
