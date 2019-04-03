//! An efficient way of drawing images composed out of tiles from a spritesheet,
//! such as oldschool RPG maps.
//!
//! Includes a loader for the `tiled` map editor format.
//! It doesn't use all of the `tiled` map format's features though.
//! Notably: Only one TileSet is allowed, the TileSet may have only
//! one Image, properties and such are not used...
//!
//! You CAN draw directly from a `tiled` map, but this does a lot
//! of the annoying work of layering and coordinate transformation
//! for you.  `ggez` uses float indices for rect's while Tiled uses
//! pixel offsets, this tries to cull out tiles that are entirely
//! obscured by other tiles, etc.

use std::collections::HashMap;

use ggez;
use ggez::graphics::{self, spritebatch::SpriteBatch};
use tiled;

/// Newtype struct for a tile ID.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TileId(pub usize);

/// A struct containing info on how to draw a tile.
/// Having this does make life easier, honest.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tile {
    /// The source rect of the tile in the image.
    rect: graphics::Rect,
    /// Whether or not the tile entirely shadows the one
    /// beneath it.
    opaque: bool,
}

impl Tile {
    pub fn new(rect: graphics::Rect, opaque: bool) -> Self {
        Self { rect, opaque }
    }
}

/*
Really not sure if we want this to be its own public type;
it's sorta dubiously useful but you have to have something
like it eventually...
For now we'll just keep things as low-level as possible.

/// A lookup table from `TileId` to `Tile`.
/// ...kinda don't like the overloaded use of the term `Map` here,
/// but it is just a `HashMap` internally, so.
pub struct TileMap {
    tiles: HashMap<TileId, Tile>,
}

impl TileMap {
    fn new() -> Self {
        TileMap {
            tiles: HashMap::new(),
        }
    }
}
*/

/// A single layer in the map.
/// Each item is a source rect, or None
/// if there is nothing to be drawn for that location,
/// which makes life a lot simpler when drawing layered maps.
///
/// Tiles are stored in row-major order.
pub struct Layer {
    pub tiles: Vec<Option<TileId>>,
}

impl Layer {
    /// Returns the tile ID at the given coordinate.
    fn get_tile(&self, x: usize, y: usize, width: usize) -> Option<TileId> {
        let offset = (y * width) + x;
        self.tiles[offset]
    }
}

/// A collection of layers, all the same size
/// and all using the same `Image`.
///
/// This is intended to be a graphical artifact, not
/// a gameplay one.  If you need collision detection or such,
/// have another structure alongside this one.  If you need
/// multiple layers with different source images, use a stack
/// of these.
///
/// Currently there's no way to animate this, though it should be
/// added in the future.  An easy and efficient option would be making
/// multiple entire Image's and having this able to flip between them.
pub struct Map {
    pub layers: Vec<Layer>,
    /// Width, in tiles
    pub width: usize,
    /// Height, in tiles
    pub height: usize,

    /// A map from arbitrary ID's to `Tile`'s.
    ///
    /// Having this separate makes life a lot easier 'cause
    /// we only have to do math once.
    pub tile_map: HashMap<TileId, Tile>,

    batch: SpriteBatch,
}

impl Map {
    /// Low-level constructor for creating a `Map`.  You give it a set
    /// of layers and a `TileMap` you have already created.
    pub fn new(
        width: usize,
        height: usize,
        layers: Vec<Vec<Option<TileId>>>,
        image: graphics::Image,
        tile_map: HashMap<TileId, Tile>,
    ) -> Self {
        let layers: Vec<Layer> = layers.into_iter().map(|l| {
            // Ensure all layers are the right size.
            assert_eq!(width*height, l.len());
            Layer { tiles: l }
        }).collect();
        let mut s = Self {
            layers,
            width,
            height,

            tile_map,
            batch: SpriteBatch::new(image),
        };
        s.batch_layers();
        s
    }

    /// Construct a map from a `tiled::Map`.
    /// Needs a function that will take an image source path and create/fetch
    /// a `ggez::graphics::Image` from it.
    pub fn from_tiled(t: tiled::Map, image_callback: &dyn Fn(&str) -> graphics::Image) -> Self {
        let width = t.width as usize;
        let height = t.height as usize;
        if t.tilesets.len() != 1 {
            panic!("Invalid number of tilesets: {}", t.tilesets.len());
        }
        let tileset = &t.tilesets[0];
        if tileset.images.len() != 1 {
            panic!(
                "Invalid number of images in tileset: {}",
                tileset.images.len()
            );
        }
        let image_str = &tileset.images[0].source;
        let image = image_callback(image_str);
        let image_rect = image.dimensions();

        // Translation table from the file's global id's to our TileId's.
        // Probably technically unnecessary but...
        let mut gid_to_tileid = HashMap::new();
        let mut tile_map = HashMap::new();

        // Turn a `tiled::TileSet` into a hashmap of our own `Tile` types.
        for (i, t) in tileset.tiles.iter().enumerate() {
            let id = TileId(i);
            let tile_width = tileset.tile_width as f32 / image_rect.w;
            let tile_height = tileset.tile_height as f32 / image_rect.h;
            // TODO: Spacing and margin

            // TODO: Actually resolve the X and Y's, it's not clear how
            // to do that just from the `tiled` docs.  I recall it needing
            // some experimentation.
            let tile_rect = graphics::Rect {
                x: 0.,
                y: 0.,
                w: tile_width,
                h: tile_height,
            };
            let tile = Tile {
                rect: tile_rect,
                /// TODO: Pull from an attr or something?
                opaque: true,
            };
            tile_map.insert(id, tile);
            gid_to_tileid.insert(t.id, id);
        }

        // Great, now we have a tile set, we can translate
        // the layers.
        let layers: Vec<Layer> = t
            .layers
            .iter()
            .map(|layer| {
                // TODO: Figure out how Tiled stores empty tiles.
                // IIRC they're gid 0 or something like that but we
                // need to verify.
                let tiles: Vec<Option<TileId>> = layer
                    .tiles
                    .iter()
                    .flatten()
                    .map(|gid| Some(*gid_to_tileid.get(gid).expect("GID not found?!")))
                    .collect();
                Layer { tiles }
            })
            .collect();

        let batch = SpriteBatch::new(image);
        let mut s = Self {
            layers,
            tile_map,
            width,
            height,
            batch,
        };
        s.batch_layers();
        s
    }

    /// Goes through all the `Layer`'s in this image and enters them
    /// into the SpriteBatch, replacing whatever's already there.
    fn batch_layers(&mut self) {
        self.batch.clear();
        for x in 0..self.width {
            for y in 0..self.height {
                let first_opaque_layer = self.first_opaque_layer_at(x, y);
                for layer in &self.layers[first_opaque_layer..] {
                    if let Some(tile_idx) = layer.get_tile(x, y, self.width) {
                        let tile = self.tile_map.get(&tile_idx).expect("Invalid tile ID!");
                        let src_rect = tile.rect;
                        let dest_pt: crate::Point2 =
                            euclid::point2(src_rect.w * (x as f32), src_rect.h * (y as f32));
                        let _ = self
                            .batch
                            .add(graphics::DrawParam::default().src(src_rect).dest(dest_pt));
                    }
                }
            }
        }
    }

    /// Walk down the stack of `Layer`'s at a coordinate,
    /// finding the first one with a tile at that location marked opaque.
    /// Returns the layer index of the opaque tile.
    ///
    /// If no layers are  opaque, returns 0, meaning the bottom layer.  This
    /// should maybe be an error though, since you generally don't want to to
    /// see through your map.  Maybe a debug flag or warning?
    ///
    /// Panics if no layers exist.
    fn first_opaque_layer_at(&self, x: usize, y: usize) -> usize {
        assert!(self.layers.len() > 0);
        for i in (0..self.layers.len()).rev() {
            if let Some(tile_idx) = self.layers[i].get_tile(x, y, self.width) {
                let tile = self.tile_map.get(&tile_idx).expect("Invalid tile ID!");
                if tile.opaque {
                    return i;
                }
                // Tile is transparent, continue
            }
            // No tile at that coordinate, continue
        }
        return 0;
    }
}

impl graphics::Drawable for Map {
    fn draw(&self, ctx: &mut ggez::Context, param: graphics::DrawParam) -> ggez::GameResult {
        self.batch.draw(ctx, param)
    }

    /// This is kinda odd 'cause tiles don't *strictly* all need to be the same size...
    /// TODO: Find out if Tiled can ever create ones that aren't.
    fn dimensions(&self, ctx: &mut ggez::Context) -> Option<graphics::Rect> {
        self.batch.dimensions(ctx)
    }

    fn set_blend_mode(&mut self, mode: Option<graphics::BlendMode>) {
        self.batch.set_blend_mode(mode);
    }
    fn blend_mode(&self) -> Option<graphics::BlendMode> {
        self.batch.blend_mode()
    }
}

// TODO: Unit tests.  We need a simple Tiled map to test with.
