
use std::collections::HashMap;
use ggez;

/// Describes the layout of characters in your
/// bitmap font.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextMap {
    map: HashMap<char, ggez::Rect>,
}

impl TextMap {
    /// Creates a new `TextMap` from a uniform grid of
    /// sprites.  Takes the number of sprites wide and
    /// tall that the bitmap should be, and a string
    /// describing the characters in the map... in order,
    /// left to right, top to bottom.
    /// 
    /// The characters do not necessarily need to fill
    /// the entire image.  ie, if your image is 16x16 glyphs
    /// for 256 total, and you only use the first 150 of them,
    /// that's fine.
    /// 
    /// The floating point math involved should always be
    /// exact for `Image`'s and sprites with a resolution 
    /// that is a power of two, I think.
    fn from_grid(mapping: &str, width: usize, height: usize) -> Self {
        // Assert the given width and height can fit the listed characters.
        let num_chars = mapping.chars.count();
        assert!(num_chars <= width * height);
        let rect_width = 1.0 / (width as f32);
        let rect_height = 1.0 / (height as f32);
        let mut map = HashMap::with_capacity(num_chars);
        let mut current_x = 0;
        let mut current_y = 0;
        for c in mapping.chars() {
            let x_offset = current_x as f32 * rect_width;
            let y_offset = current_y as f32 * rect_height;
            let char_rect = ggez::Rect {
                x: x_offset,
                y: y_offset,
                w: rect_width,
                h: rect_height;
            };
            map.insert(c, char_rect);
            current_x = (current_x + 1) % width;
            if current_x == 0 {
                current_y += 1;
            }
        }

        Self {
            map,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitmapFont {
    bitmap: ggez::graphics::Image,
    batch: ggez::graphics::SpriteBatch,
    map: TextMap,
}

impl BitmapFont {

}