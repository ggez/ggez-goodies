//! A camera object for ggez.
//! Currently ggez has no actual global camera state to use,
//! so this really just does the coordinate transforms for you.
//!
//! Basically it translates ggez's integer-valued coordinate
//! system with the origin at the top-left and Y increasing
//! downward, to a float-valued coordinate system with the
//! origin at the center of the screen and Y increasing upward.
//!
//! Because that makes sense, darn it.

use ggez;
use ggez::GameResult;
use ggez::graphics;
use ggez::graphics::Drawable;
use na;
use super::{Point2, Vector2};

// Hmm.  Could, instead, use a 2d transformation
// matrix, or create one of such.
pub struct Camera {
    screen_width: u32,
    screen_height: u32,
    view_width: f64,
    view_height: f64,
    view_center: Vector2,
}

impl Camera {
    pub fn new(screen_width: u32, screen_height: u32, view_width: f64, view_height: f64) -> Self {
        Camera {
            screen_width: screen_width,
            screen_height: screen_height,
            view_width: view_width,
            view_height: view_height,
            view_center: na::zero(),
        }
    }

    pub fn move_by(&mut self, by: Vector2) {
        self.view_center += by;
    }

    pub fn move_to(&mut self, to: Vector2) {
        self.view_center = to;
    }

    /// Translates a point in world-space to a point in
    /// screen-space.
    ///
    /// Does not do any clipping or anything, since it does
    /// not know how large the thing that might be drawn is;
    /// that's not its job.
    pub fn world_to_screen_coords(&self, from: Vector2) -> (i32, i32) {
        let sw = self.screen_width as f64;
        let sh = self.screen_height as f64;
        let pixels_per_unit_x = sw / self.view_width;
        let pixels_per_unit_y = sh / self.view_height;
        let scale_vec = Vector2::new(pixels_per_unit_x, pixels_per_unit_y);


        let view_offset = from - self.view_center;
        let view_scale = view_offset * scale_vec;


        let x = view_scale.x + sw / 2.0;
        let y = sh - (view_scale.y + sh / 2.0);
        (x as i32, y as i32)



        // let x = from.x + sw / 2.0;
        // let y = sh - (from.y + sh / 2.0);
        // (x as i32, y as i32)
    }


    pub fn screen_to_world_coords(&self, from: (i32, i32)) -> Vector2 {
        let (sx, sy) = from;
        na::zero()
    }

    pub fn location(&self) -> Vector2 {
        self.view_center
    }

    fn calculate_dest_rect(&self, location: Vector2, dst_size: (u32, u32)) -> graphics::Rect {
        let (sx, sy) = self.world_to_screen_coords(location);
        let (sw, sh) = dst_size;
        graphics::Rect::new(sx, sy, sw, sh)
    }
}

pub trait CameraDraw
    where Self: graphics::Drawable
{
    fn draw_ex_camera(&mut self,
                      camera: &Camera,
                      location: Vector2,
                      context: &mut ggez::Context,
                      src: Option<graphics::Rect>,
                      dst_size: (u32, u32),
                      angle: f64,
                      center: Option<graphics::Point>,
                      flip_horizontal: bool,
                      flip_vertical: bool)
                      -> GameResult<()> {
        let dest_rect = camera.calculate_dest_rect(location, dst_size);
        self.draw_ex(context,
                     src,
                     Some(dest_rect),
                     angle,
                     center,
                     flip_horizontal,
                     flip_vertical)
    }


    fn draw_camera(&mut self,
                   camera: &Camera,
                   location: Vector2,
                   context: &mut ggez::Context,
                   src: Option<graphics::Rect>,
                   dst_size: (u32, u32))
                   -> GameResult<()> {

        let dest_rect = camera.calculate_dest_rect(location, dst_size);
        self.draw(context, src, Some(dest_rect))
    }
}


impl<T> CameraDraw for T where T: graphics::Drawable {}
