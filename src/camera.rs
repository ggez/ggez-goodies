//! A camera object for ggez.
//! Currently ggez has no actual global camera state to use,
//! so this really just does the coordinate transforms for you.

use ggez;
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

    pub fn world_to_screen_coords(&self, from: Vector2) -> (u32, u32) {
        let width = self.screen_width as f64;
        let height = self.screen_height as f64;
        let x = from.x + width / 2.0;
        let y = height - (from.y + height / 2.0);
        (x as u32, y as u32)
    }


    pub fn screen_to_world_coords(&self, from: (u32, u32)) -> Vector2 {
        let (sx, sy) = from;
        na::zero()
    }
}
