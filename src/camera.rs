//! A camera object for ggez.
//! Currently ggez has no actual global camera state to use,
//! so this really just does the coordinate transforms for you.
//!
//! Basically it translates ggez's coordinate system with the origin
//! at the top-left and Y increasing downward to a coordinate system
//! with the origin at the center of the screen and Y increasing
//! upward.
//!
//! Because that makes sense, darn it.
//!
//! However, does not yet do any actual camera movements like
//! easing, pinning, etc.
//! But a great source for how such things work is this:
//! http://www.gamasutra.com/blogs/ItayKeren/20150511/243083/Scroll_Back_The_Theory_and_Practice_of_Cameras_in_SideScrollers.php

// TODO: Debug functions to draw world and camera grid!

use ggez;
use ggez::graphics;
use ggez::mint::{Point2, Vector2};
use ggez::GameResult;

// Hmm.  Could, instead, use a 2d transformation
// matrix, or create one of such.
pub struct Camera {
    screen_size: Vector2<f32>,
    view_size: Vector2<f32>,
    view_center: Point2<f32>,
}

impl Camera {
    pub fn new(screen_width: u32, screen_height: u32, view_width: f32, view_height: f32) -> Self {
        let screen_size = Vector2 {
            x: screen_width as f32,
            y: screen_height as f32,
        };
        let view_size = Vector2 {
            x: view_width as f32,
            y: view_height as f32,
        };
        Camera {
            screen_size: screen_size,
            view_size: view_size,
            view_center: Point2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn move_by(&mut self, by: Vector2<f32>) {
        self.view_center.x += by.x;
        self.view_center.y += by.y;
    }

    pub fn move_to(&mut self, to: Point2<f32>) {
        self.view_center = to;
    }

    /// Translates a point in world-space to a point in
    /// screen-space.
    ///
    /// Does not do any clipping or anything, since it does
    /// not know how large the thing that might be drawn is;
    /// that's not its job.
    pub fn world_to_screen_coords(&self, from: Point2<f32>) -> (i32, i32) {
        let pixels_per_unit = Vector2 {
            x: self.screen_size.x / self.view_size.x,
            y: self.screen_size.y / self.view_size.y,
        };
        let view_offset = Vector2 {
            x: from.x - self.view_center.x,
            y: from.y - self.view_center.y,
        };
        let view_scale = Vector2 {
            x: view_offset.x * pixels_per_unit.x,
            y: view_offset.y * pixels_per_unit.y,
        };

        let x = view_scale.x + self.screen_size.x / 2.0;
        let y = self.screen_size.y - view_scale.y + self.screen_size.y / 2.0;
        (x as i32, y as i32)
    }

    // p_screen = max_p - p + max_p/2
    // p_screen - max_p/2 = max_p - p
    // p_screen - max_p/2 + max_p = -p
    // -p_screen - max_p/2 + max_p = p
    pub fn screen_to_world_coords(&self, from: (i32, i32)) -> Point2<f32> {
        let (sx, sy) = from;
        let sx = sx as f32;
        let sy = sy as f32;
        let flipped_x = sx - self.screen_size.x / 2.0;
        let flipped_y = -sy + self.screen_size.y / 2.0;
        let screen_coords = Vector2 {
            x: flipped_x,
            y: flipped_y,
        };
        let units_per_pixel = Vector2 {
            x: self.view_size.x / self.screen_size.x,
            y: self.view_size.y / self.screen_size.y,
        };
        let view_scale = Vector2 {
            x: screen_coords.x * units_per_pixel.x,
            y: screen_coords.y * units_per_pixel.y,
        };
        let view_offset = Point2 {
            x: self.view_center.x + view_scale.y,
            y: self.view_center.y + view_scale.y,
        };

        view_offset
    }

    pub fn location(&self) -> Point2<f32> {
        self.view_center
    }

    fn calculate_dest_point(&self, location: Point2<f32>) -> Point2<f32> {
        let (sx, sy) = self.world_to_screen_coords(location);
        Point2 {
            x: sx as f32,
            y: sy as f32,
        }
    }
}

pub trait CameraDraw
where
    Self: graphics::Drawable,
{
    fn draw_ex_camera(
        &self,
        camera: &Camera,
        ctx: &mut ggez::Context,
        p: ggez::graphics::DrawParam,
    ) -> GameResult<()> {
        let dest = camera.calculate_dest_point(p.dest);
        let mut my_p = p;
        my_p.dest = dest;
        self.draw(ctx, my_p)
    }

    fn draw_camera(
        &self,
        camera: &Camera,
        ctx: &mut ggez::Context,
        dest: Point2<f32>,
        rotation: f32,
    ) -> GameResult<()> {
        let dest = camera.calculate_dest_point(dest);
        let mut draw_param = ggez::graphics::DrawParam::default();
        draw_param.dest = dest;
        draw_param.rotation = rotation;
        self.draw(ctx, draw_param)
    }
}

impl<T> CameraDraw for T where T: graphics::Drawable {}

#[cfg(test)]
mod tests {
    use super::*;
    use ggez::nalgebra::{Point2, Vector2};

    #[test]
    fn test_coord_round_trip() {
        let mut c = Camera::new(640, 480, 40.0, 30.0);
        let p1 = (200, 300);
        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Point2::new(-7.5, -3.75));
            let p1_screen = c.world_to_screen_coords(p1_world);
            assert_eq!(p1, p1_screen);
        }

        let p2 = Point2::new(20.0, 10.0);
        {
            let p2_screen = c.world_to_screen_coords(p2);
            assert_eq!(p2_screen, (640, 80));
            let p2_world = c.screen_to_world_coords(p2_screen);
            assert_eq!(p2_world, p2);
        }

        c.move_to(Point2::new(5.0, 5.0));

        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Point2::new(-2.5, 1.25));
            let p1_screen = c.world_to_screen_coords(p1_world);
            assert_eq!(p1, p1_screen);
        }
        {
            let p2_screen = c.world_to_screen_coords(p2);
            assert_eq!(p2_screen, (560, 160));
            let p2_world = c.screen_to_world_coords(p2_screen);
            assert_eq!(p2_world, p2);
        }
    }
}
