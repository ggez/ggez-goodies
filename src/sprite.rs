//! Sprites!
//! We want atlasing, flipbook animations, layering, tilemaps...

use ggez::graphics;

/// An object that contains metadata on an image atlas.
/// Does it contain the image itself or not?
struct Atlas {
    source: graphics::Image,
    /// The number of sub-images across 
    width: u32,
    /// The number of sub-images high
    height: u32,
}

struct Sprite<'a> {
    atlas: &'a Atlas,
    index: u32,
    layer: u32,
}

/// A `SpriteManager` is in charge of doing all sprite drawing.
/// It manages `Atlas`es, `Sprite`s, and so on.
/// When you tell it to draw, it will draw all sprites,
/// doing layering and such.
struct SpriteManager<'a> {
    atlas: Atlas,
    sprites: Vec<Sprite<'a>>,
}
