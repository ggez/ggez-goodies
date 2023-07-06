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
use ggez::graphics::Transform;
use ggez::mint;
use ggez::GameResult;
use nalgebra_glm::Vec2;

// Used for mint interoperability.
struct Vector2(Vec2);
struct MintPoint2(mint::Point2<f32>);

impl From<MintPoint2> for Vec2 {
    fn from(val: MintPoint2) -> Self {
        Vec2::new(val.0.x, val.0.y)
    }
}

impl From<Vector2> for mint::Point2<f32> {
    fn from(val: Vector2) -> Self {
        mint::Point2 {
            x: val.0.x,
            y: val.0.y,
        }
    }
}

/// The actual camera.  Stores the screen size, where it's looking, and how big the POV is.
pub struct Camera {
    screen_size: Vec2,
    view_size: Vec2,
    view_center: Vec2,
}

impl Camera {
    pub fn new(screen_width: u32, screen_height: u32, view_width: f32, view_height: f32) -> Self {
        let screen_size = Vec2::new(screen_width as f32, screen_height as f32);
        let view_size = Vec2::new(view_width, view_height);
        Camera {
            screen_size,
            view_size,
            view_center: Vec2::new(0.0, 0.0),
        }
    }

    pub fn move_by(&mut self, by: Vec2) {
        self.view_center.x += by.x;
        self.view_center.y += by.y;
    }

    pub fn move_to(&mut self, to: Vec2) {
        self.view_center = to;
    }

    /// Translates a point in world-space to a point in
    /// screen-space.
    ///
    /// Does not do any clipping or anything, since it does
    /// not know how large the thing that might be drawn is;
    /// that's not its job.
    pub fn world_to_screen_coords(&self, from: Vec2) -> (i32, i32) {
        let pixels_per_unit = self.screen_size.component_div(&self.view_size);
        let view_offset = from - self.view_center;
        let view_scale = view_offset.component_mul(&pixels_per_unit);

        let x = view_scale.x + self.screen_size.x / 2.0;
        let y = self.screen_size.y - (view_scale.y + self.screen_size.y / 2.0);
        (x as i32, y as i32)
    }

    // p_screen = max_p - p + max_p/2
    // p_screen - max_p/2 = max_p - p
    // p_screen - max_p/2 + max_p = -p
    // -p_screen - max_p/2 + max_p = p
    pub fn screen_to_world_coords(&self, from: (i32, i32)) -> Vec2 {
        let (sx, sy) = from;
        let sx = sx as f32;
        let sy = sy as f32;
        let flipped_x = sx - (self.screen_size.x / 2.0);
        let flipped_y = -sy + self.screen_size.y / 2.0;
        let screen_coords = Vec2::new(flipped_x, flipped_y);
        let units_per_pixel = self.view_size.component_div(&self.screen_size);
        let view_scale = screen_coords.component_mul(&units_per_pixel);
        self.view_center + view_scale
    }

    pub fn location(&self) -> Vec2 {
        self.view_center
    }

    fn calculate_dest_point(&self, location: Vec2) -> Vec2 {
        let (sx, sy) = self.world_to_screen_coords(location);
        Vec2::new(sx as f32, sy as f32)
    }
}

pub trait CameraDraw
where
    Self: graphics::Drawable,
{
    fn draw_ex_camera(
        &self,
        camera: &Camera,
        canvas: &mut ggez::graphics::Canvas,
        p: ggez::graphics::DrawParam,
    ) -> GameResult<()> {
        if let Transform::Values { dest, .. } = p.transform {
            let my_dest = camera.calculate_dest_point(MintPoint2(dest).into());
            let my_p = p.dest(Vector2(my_dest));
            self.draw(canvas, my_p);
            return Ok(());
        }
        Err(ggez::GameError::CustomError(
            "Failed to draw to camera".to_string(),
        ))
    }

    fn draw_camera(
        &self,
        camera: &Camera,
        canvas: &mut ggez::graphics::Canvas,
        dest: Vec2,
        rotation: f32,
    ) -> GameResult<()> {
        let dest = camera.calculate_dest_point(dest);
        let draw_param = ggez::graphics::DrawParam::default()
            .dest(Vector2(dest))
            .rotation(rotation);
        self.draw(canvas, draw_param);
        Ok(())
    }
}

impl<T> CameraDraw for T where T: graphics::Drawable {}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::Vec2;

    #[test]
    fn test_coord_round_trip() {
        let mut c = Camera::new(640, 480, 40.0, 30.0);
        let p1 = (200, 300);
        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Vec2::new(-7.5, -3.75));
            let p1_screen = c.world_to_screen_coords(p1_world);
            assert_eq!(p1, p1_screen);
        }

        let p2 = Vec2::new(20.0, 10.0);
        {
            let p2_screen = c.world_to_screen_coords(p2);
            assert_eq!(p2_screen, (640, 80));
            let p2_world = c.screen_to_world_coords(p2_screen);
            assert_eq!(p2_world, p2);
        }

        c.move_to(Vec2::new(5.0, 5.0));

        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Vec2::new(-2.5, 1.25));
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
