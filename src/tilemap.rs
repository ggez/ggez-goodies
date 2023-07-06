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
use std::fmt::Debug;

use ggez::context::Has;
use ggez::graphics::{self, Drawable, GraphicsContext, Mesh, MeshData};
use ggez::{self, Context};
pub use tiled;

/// Newtype struct for a tile ID.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TileId(pub usize);

/// A struct containing info on how to draw a tile.
/// Having this rather than just a bare `Rect` or something
/// does make life easier, honest.
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

/// A collection of `Tile` definitions and the `Image` they refer to.
#[derive(Clone, Debug)]
pub struct Tileset {
    pub first_gid: usize,
    pub tileset: HashMap<TileId, Tile>,
    image: graphics::Image,
}

impl Tileset {
    /// Turn a `tiled::Tileset` into a hashmap of our own `Tile` types.
    /// Having our own types for all the `Tiled` types is necessary for
    /// coordinate translation and such, annoyingly.
    pub fn from_tiled(tset: &tiled::Tileset, image: graphics::Image, ctx: &Context) -> Self {
        let mut tileset = HashMap::new();

        let image_rect = image.dimensions(ctx).unwrap_or_default();
        let image_widthi = image_rect.w as u32;
        let image_heighti = image_rect.h as u32;
        let tile_width = tset.tile_width as f32 / image_rect.w;
        let tile_height = tset.tile_height as f32 / image_rect.h;
        let first_gid = tset.first_gid as usize;

        // Calculate number of tiles.
        // Any fractions just get truncated off; Tiled 1.2 does the same thing.
        let tiles_per_row = image_widthi / tset.tile_width;
        let rows = image_heighti / tset.tile_height;
        let tile_count = tiles_per_row * rows;

        // Iterate over the tiles that actually have properties and such, and save them.
        // TODO:
        // Figure out gid translations better; I think it's tile.id - tset.first_gid ?
        // Right now we just assume that gid's start from 1.
        // Decide what to do with tile properties.
        /*
                for (i, t) in tset.tiles.iter().enumerate() {
                    //let id = TileId();
                    //gid_to_tileid.insert(t.id, id);
                }
        */
        for i in 0..tile_count {
            // tiled tile ID's seem to start at 1.
            let id = TileId(i as usize + 1);
            let x = i % tiles_per_row;
            let y = i / tiles_per_row;
            // TODO: Spacing and margin

            // Actually translate the X's and Y's to offsets, it's not
            // clear how to do that just from the `tiled` docs.
            // Looking at the file, it looks like it just counts from
            // the top-left corner, it knows the dimensions of the
            // image and so just uses the dimensions of the tiles to
            // calculate offsets.  It actually omits tiles that don't
            // have anything EXCEPT an offset, it appears.
            let tile_rect = graphics::Rect {
                x: x as f32 * tile_width,
                y: y as f32 * tile_height,
                w: tile_width,
                h: tile_height,
            };
            let tile = Tile {
                rect: tile_rect,
                /// TODO: Pull from an attr or something?
                opaque: true,
            };
            tileset.insert(id, tile);
        }

        Self {
            tileset,
            image,
            first_gid,
        }
    }

    /// TODO
    fn translate_gid(&self, gid: u32) -> TileId {
        TileId(gid as usize)
    }

    fn get(&self, id: TileId) -> (Option<&Tile>, bool, bool, bool) {
        let id = id.0;
        let (hflip, vflip, dflip) = (id & 1 << 31 != 0, id & 1 << 30 != 0, id & 1 << 29 != 0); //Get orientation flags from id.
        let id = TileId(id & !(7 << 29)); //Discard flag bits
        (self.tileset.get(&id), hflip, vflip, dflip)
    }
}

/// A single layer in the map.
/// Each item is a source rect, or None
/// if there is nothing to be drawn for that location,
/// which makes life a lot simpler when drawing layered maps.
///
/// Tiles are stored in row-major order.
#[derive(Clone, Debug, PartialEq)]
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
/// and all using the same `Tileset`.
///
/// This is intended to be a graphical artifact, not
/// a gameplay one.  If you need collision detection or such,
/// have another structure alongside this one.  If you need
/// multiple layers with different source images, use a stack
/// of these.
///
/// Currently there's no way to animate this, though it should be
/// added in the future.  An easy and efficient option would be making
/// multiple entire `Tileset`'s and having this able to flip between them.
/// Right now though it only contains a single `Tileset`.
#[derive(Clone, Debug)]
pub struct Map {
    pub layers: Vec<Layer>,
    /// Width, in tiles
    pub width: usize,
    /// Height, in tiles
    pub height: usize,

    /// Tile width, in screen units
    pub tile_width: f32,
    /// Tile height, in screen units
    pub tile_height: f32,

    /// A map from arbitrary ID's to `Tile`'s.
    pub tileset: Tileset,

    /// The constructed mesh of tiles.
    mesh: graphics::Mesh,
}

impl Map {
    /// Low-level constructor for creating a `Map`.  You give it a set
    /// of layers and a `TileMap` you have already created.
    pub fn new(
        ctx: &mut ggez::Context,
        width: usize,
        height: usize,
        tile_width: f32,
        tile_height: f32,
        layers: Vec<Vec<Option<TileId>>>,
        tileset: Tileset,
    ) -> Self {
        let layers: Vec<Layer> = layers
            .into_iter()
            .map(|l| {
                // Ensure all layers are the right size.
                assert_eq!(width * height, l.len());
                Layer { tiles: l }
            })
            .collect();
        // Dummy mesh, replaced by the `batch_layers()` call.
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, 100.0, 100.0),
            graphics::Color::WHITE,
        )
        .unwrap();
        let mut s = Self {
            layers,
            width,
            height,

            tile_width,
            tile_height,
            tileset,
            mesh,
        };
        s.batch_layers(ctx);
        s
    }

    /// Construct a map from a `tiled::Map`.
    /// Needs a function that will take an image source path and create/fetch
    /// a `ggez::graphics::Image` from it.
    pub fn from_tiled(
        ctx: &mut ggez::Context,
        t: tiled::Map,
        image_callback: &mut dyn FnMut(&mut ggez::Context, &str) -> graphics::Image,
    ) -> Self {
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

        let tile_width = tileset.tile_width as f32;
        let tile_height = tileset.tile_height as f32;
        let image_str = &tileset.images[0].source;
        let image = image_callback(ctx, image_str);
        let tileset = Tileset::from_tiled(&t.tilesets[0], image, ctx);

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
                    .map(|gid| Some(tileset.translate_gid(*gid)))
                    .collect();
                Layer { tiles }
            })
            .collect();

        // Dummy mesh
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, 100.0, 100.0),
            graphics::Color::WHITE,
        )
        .unwrap();

        let mut s = Self {
            layers,
            tileset,
            width,
            height,
            tile_width,
            tile_height,
            mesh,
        };
        s.batch_layers(ctx);
        s
    }

    /// Goes through all the `Layer`'s in this image and enters them
    /// into the SpriteBatch, replacing whatever's already there.
    fn batch_layers(&mut self, ctx: &mut ggez::Context) {
        // TODO: Now that I think of it, this probably doesn't handle
        // layers properly.  If we try to make two quads at the same spot
        // they won't draw "in order" as `SpriteBatch`'s would, they'll
        // just z-fight.
        //
        // What we currently call a `Map` should become a `Layer`.
        let mut verts: Vec<graphics::Vertex> = vec![];
        let mut indices = vec![];
        let mut idx = 0;

        for x in 0..self.width {
            for y in 0..self.height {
                let first_opaque_layer = self.first_opaque_layer_at(x, y);
                for layer in &self.layers[first_opaque_layer..] {
                    if let Some(tile_idx) = layer.get_tile(x, y, self.width) {
                        if tile_idx.0 != 0 {
                            //Continue if tile is empty.
                            let (tile, hflip, vflip, dflip) = self.tileset.get(tile_idx);
                            let tile = tile.expect("Invalid tile ID!");
                            let src_rect = tile.rect;
                            let dest_pt: crate::Point2 = euclid::point2(
                                (x as f32) * self.tile_width,
                                (y as f32) * self.tile_height,
                            );
                            let mut v = [
                                graphics::Vertex {
                                    position: [dest_pt.x, dest_pt.y],
                                    uv: [src_rect.x, src_rect.y],
                                    color: graphics::Color::WHITE.into(),
                                },
                                graphics::Vertex {
                                    position: [dest_pt.x + self.tile_width, dest_pt.y],
                                    uv: [src_rect.x + src_rect.w, src_rect.y],
                                    color: graphics::Color::WHITE.into(),
                                },
                                graphics::Vertex {
                                    position: [
                                        dest_pt.x + self.tile_width,
                                        dest_pt.y + self.tile_height,
                                    ],
                                    uv: [src_rect.x + src_rect.w, src_rect.y + src_rect.h],
                                    color: graphics::Color::WHITE.into(),
                                },
                                graphics::Vertex {
                                    position: [dest_pt.x, dest_pt.y + self.tile_height],
                                    uv: [src_rect.x, src_rect.y + src_rect.h],
                                    color: graphics::Color::WHITE.into(),
                                },
                            ];
                            if dflip {
                                //Swap uv coordinates of diagonally opposite corners to rotate texture.
                                let (v1uv, v3uv) = (v[1].uv, v[3].uv);
                                v[1].uv = v3uv;
                                v[3].uv = v1uv;
                            };
                            if hflip {
                                //Swap uv coordinates of horizontally opposite corners to flip texture horizontally.
                                let (v0uv, v1uv, v2uv, v3uv) = (v[0].uv, v[1].uv, v[2].uv, v[3].uv);
                                v[0].uv = v1uv;
                                v[1].uv = v0uv;
                                v[2].uv = v3uv;
                                v[3].uv = v2uv;
                            };
                            if vflip {
                                //Swap uv coordinates of vertically opposite corners to flip texture vertically.
                                let (v0uv, v1uv, v2uv, v3uv) = (v[0].uv, v[1].uv, v[2].uv, v[3].uv);
                                v[0].uv = v3uv;
                                v[1].uv = v2uv;
                                v[2].uv = v1uv;
                                v[3].uv = v0uv;
                            };

                            verts.extend(&v);
                            // Index a quad
                            indices.extend(&[idx, idx + 1, idx + 2, idx + 2, idx + 3, idx]);
                            // indices.extend(&[idx, idx + 1, idx + 2, idx, idx + 3, idx]);
                            idx += 4;
                        }
                    }
                }
            }
        }
        // let mut mb = graphics::MeshBuilder::default();
        let mesh_data = MeshData {
            vertices: verts.as_slice(),
            indices: indices.as_slice(),
        };
        self.mesh = Mesh::from_data(ctx, mesh_data);
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
        assert!(!self.layers.is_empty());
        for i in (0..self.layers.len()).rev() {
            if let Some(tile_idx) = self.layers[i].get_tile(x, y, self.width) {
                if tile_idx.0 != 0 {
                    let tile = self.tileset.get(tile_idx).0.expect("Invalid tile ID!");
                    if tile.opaque {
                        return i;
                    }
                    // Tile is transparent, continue
                }
                //Tile is empty, continue
            }
            // No tile at that coordinate, continue
        }
        0
    }
}

impl graphics::Drawable for Map {
    fn draw(&self, canvas: &mut ggez::graphics::Canvas, param: impl Into<graphics::DrawParam>) {
        canvas.draw_textured_mesh(self.mesh.clone(), self.tileset.image.clone(), param);
    }

    /// This is kinda odd 'cause tiles don't *strictly* all need to be the same size...
    /// TODO: Find out if Tiled can ever create ones that aren't.
    fn dimensions(&self, gfx: &impl Has<GraphicsContext>) -> Option<graphics::Rect> {
        self.mesh.dimensions(gfx)
    }
}

// TODO: Unit tests.
