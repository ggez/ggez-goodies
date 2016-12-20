//! A camera object for ggez.
//! Currently ggez has no actual global camera state to use,
//! so this really just does the coordinate transforms for you.

use ggez;
use ggez::GameResult;
use ggez::graphics;
use ggez::graphics::Drawable;
use na;
use super::{Point2, Vector2};
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

    pub fn world_to_screen_coords(&self, from: Vector2) -> (i32, i32) {
        let width = self.screen_width as f64;
        let height = self.screen_height as f64;
        let x = from.x + width / 2.0;
        let y = height - (from.y + height / 2.0);
        (x as i32, y as i32)
    }


    pub fn screen_to_world_coords(&self, from: (i32, i32)) -> Vector2 {
        let (sx, sy) = from;
        na::zero()
    }

    fn calculate_dest_rect(&self, location: Vector2, dst_size: (u32, u32)) -> graphics::Rect {
        let (sx, sy) = self.world_to_screen_coords(location);
        let (sw, sh) = dst_size;
        graphics::Rect::new(sx, sy, sw, sh)
    }
}

trait CameraDraw
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
