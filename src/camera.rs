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
use ggez::GameResult;
use ggez::graphics;
use {Point2, Vector2, Matrix3, Similarity2, Translation2, Projective2};
use na::UnitComplex;

// Now uses Similarity and Projective transforms
pub struct Camera {
    transform: Similarity2,
    screen_transform: Projective2,
    zoom: f64
}

impl Camera {
    pub fn new(screen_width: u32, screen_height: u32, view_width: f64, view_height: f64) -> Self {
        let screen_size = Vector2::new(screen_width as f64, screen_height as f64);
        let view_size = Vector2::new(view_width as f64, view_height as f64);
        let units_per_pixel = view_size.component_div(&screen_size);
        // Similarities only support uniform scaling
        assert_eq!(units_per_pixel.x, units_per_pixel.y);
        let transform = Similarity2::new(Vector2::new(0.0, 0.0), 0.0, units_per_pixel.x);
        let screen_transform_matrix = Matrix3::new(1.0,  0.0, screen_size.x / 2.0,
                                                  0.0, -1.0, screen_size.y / 2.0,
                                                  0.0,  0.0, 1.0);
        let screen_transform = Projective2::from_matrix_unchecked(screen_transform_matrix);
        Camera {
            transform,
            screen_transform,
            zoom: 1.0
        }
    }

    pub fn move_by_global(&mut self, by: Vector2) {
        self.transform.append_translation_mut(&Translation2::from_vector(by));
    }

    pub fn move_by_local(&mut self, by: Vector2) {
        let vec = self.transform.isometry.rotation * by;
        self.move_by_global(vec);
    }

    fn ease_in_out_cub(start: f64, change: f64, t: f64) -> f64 {
        let t = t * 2.0;
        if t < 1.0 {
            change/2.0*t*t*t + start
        } else {
            let t = t - 2.0;
            change/2.0*(t*t*t + 2.0) + start
        }
    }

    fn ease_in_out_quad(start: f64, change: f64, t: f64) -> f64 {
        let t = t * 2.0;
        if t < 1.0 {
            change/2.0*t*t + start
        } else {
            let t = t - 1.0;
            -change/2.0 * (t*(t-2.0) - 1.0) + start
        }
    }

    pub fn move_towards_global_lerp(&mut self, to: Point2, t: f64) {
        let dif = (to - self.location()) * t;
        self.transform.append_translation_mut(&Translation2::new(dif.x, dif.y));
    }
    pub fn move_towards_local_lerp(&mut self, to: (f64, f64), t: f64) {
        let vec = (self.screen_to_world_coords(to) - self.location()) * t;
        self.transform.append_translation_mut(&Translation2::from_vector(vec));
    }

    pub fn move_towards_global_ease_cub(&mut self, to: Point2, t: f64) {
        let dif = to - self.location();
        let mut vec = Vector2::new(dif.x, dif.y);
        vec.x = Camera::ease_in_out_cub(0.0, vec.x, t);
        vec.y = Camera::ease_in_out_cub(0.0, vec.y, t);
        self.transform.append_translation_mut(&Translation2::from_vector(vec));
    }
    pub fn move_towards_local_ease_cub(&mut self, to: (f64, f64), t: f64) {
        let dif = self.screen_to_world_coords(to) - self.location();
        let mut vec = Vector2::new(dif.x, dif.y);
        vec.x = Camera::ease_in_out_cub(0.0, vec.x, t);
        vec.y = Camera::ease_in_out_cub(0.0, vec.y, t);
        self.transform.append_translation_mut(&Translation2::from_vector(vec));
    }

    pub fn move_towards_global_ease_quad(&mut self, to: Vector2, t: f64) {
        let dif = to - self.location();
        let mut vec = Vector2::new(dif.x, dif.y);
        vec.x = Camera::ease_in_out_quad(0.0, vec.x, t);
        vec.y = Camera::ease_in_out_quad(0.0, vec.y, t);
        self.transform.append_translation_mut(&Translation2::from_vector(vec));
    }
    pub fn move_towards_local_ease_quad(&mut self, to: (f64, f64), t: f64) {
        let dif = self.screen_to_world_coords(to) - self.location();
        let mut vec = Vector2::new(dif.x, dif.y);
        vec.x = Camera::ease_in_out_cub(0.0, vec.x, t);
        vec.y = Camera::ease_in_out_cub(0.0, vec.y, t);
        self.transform.append_translation_mut(&Translation2::from_vector(vec));
    }

    pub fn rotate_wrt_center_by(&mut self, by: f64) {
        self.transform.append_rotation_wrt_center_mut(&UnitComplex::new(by));
    }

    pub fn rotate_wrt_point_by(&mut self, point: Point2, by: f64) {
        self.transform.append_rotation_wrt_point_mut(&UnitComplex::new(by), &point);
    }

    pub fn move_to(&mut self, to: Vector2) {
        self.transform.isometry.translation = Translation2::from_vector(to);
    }

    pub fn zoom_wrt_center_by(&mut self, by: f64) {
        self.zoom *= by;
        self.transform.prepend_scaling_mut(1.0 / by);
    }

    pub fn zoom_wrt_world_point_by(&mut self, point: Point2, by: f64) {
        self.zoom *= by;
        let by = 1.0 / by;
        let scale_change = 1.0 - by;
        let dif = (point - self.location()) * scale_change;
        let translation = Translation2::new(dif.x, dif.y);
        self.transform.prepend_scaling_mut(by);
        self.transform.append_translation_mut(&translation);
    }

    pub fn zoom_wrt_screen_point_by(&mut self, point: (f64, f64), by: f64) {
        let world_point = Point2::origin() + self.screen_to_world_coords(point);
        self.zoom_wrt_world_point_by(world_point, by);
    }

    /// Translates a point in world-space to a point in
    /// screen-space.
    ///
    /// Does not do any clipping or anything, since it does
    /// not know how large the thing that might be drawn is;
    /// that's not its job.
    pub fn world_to_screen_coords(&self, from: Vector2) -> (f64, f64) {
        let point = Point2::from_coordinates(from);
        let camera_transform = self.transform.inverse();
        let point_camera = camera_transform * point;
        let point_screen = self.screen_transform * point_camera;
        (point_screen.x, point_screen.y)
    }


    // p_screen = max_p - p + max_p/2
    // p_screen - max_p/2 = max_p - p
    // p_screen - max_p/2 + max_p = -p
    // -p_screen - max_p/2 + max_p = p
    pub fn screen_to_world_coords(&self, from: (f64, f64)) -> Vector2 {
        let point = Point2::new(from.0 as f64, from.1 as f64);
        let point_world = self.screen_transform.inverse() * point;
        let point_camera = self.transform * point_world;
        Vector2::new(point_camera.x, point_camera.y)
    }

    pub fn location(&self) -> Vector2 {
        self.transform.isometry.translation.vector
    }

    fn calculate_dest_point(&self, location: Vector2) -> graphics::Point {
        let (sx, sy) = self.world_to_screen_coords(location);
        graphics::Point::new(sx as f32, sy as f32)
    }
}

pub trait CameraDraw
    where Self: graphics::Drawable
{
    fn draw_ex_camera(&self,
                      camera: &Camera,
                      ctx: &mut ggez::Context,
                      p: ggez::graphics::DrawParam)
                      -> GameResult<()> {
        let dest = Vector2::new(p.dest.x as f64, p.dest.y as f64);
        let dest = camera.calculate_dest_point(dest);
        let mut my_p = p;
        my_p.dest = dest;
        my_p.rotation = my_p.rotation + camera.transform.isometry.rotation.angle() as f32;
        let scale = camera.zoom as f32;
        my_p.scale = graphics::Point::new(scale * my_p.scale.x, scale * my_p.scale.y);
        self.draw_ex(ctx, my_p)
    }

    fn draw_camera(&self,
                   camera: &Camera,
                   ctx: &mut ggez::Context,
                   dest: ggez::graphics::Point,
                   rotation: f32)
                   -> GameResult<()> {
        let dest = Vector2::new(dest.x as f64, dest.y as f64);
        let dest = camera.calculate_dest_point(dest);
        let rotation = rotation + camera.transform.isometry.rotation.angle() as f32;
        let scale = camera.zoom as f32;
        let scale = graphics::Point::new(scale, scale);
        self.draw_ex(ctx, graphics::DrawParam{dest, rotation, scale, .. Default::default()})
    }
}


impl<T> CameraDraw for T where T: graphics::Drawable {}

#[cfg(test)]
mod tests {
    use Vector2;
    use super::*;

    #[test]
    fn test_coord_round_trip() {
        let mut c = Camera::new(640, 480, 40.0, 30.0);
        let p1 = (200.0, 300.0);
        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Vector2::new(-7.5, -3.75));
            let p1_screen = c.world_to_screen_coords(p1_world);
            assert_eq!(p1, p1_screen);
        }


        let p2 = Vector2::new(20.0, 10.0);
        {
            let p2_screen = c.world_to_screen_coords(p2);
            assert_eq!(p2_screen, (640.0, 80.0));
            let p2_world = c.screen_to_world_coords(p2_screen);
            assert_eq!(p2_world, p2);
        }

        c.move_to(Vector2::new(5.0, 5.0));

        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Vector2::new(-2.5, 1.25));
            let p1_screen = c.world_to_screen_coords(p1_world);
            assert_eq!(p1, p1_screen);
        }
        {
            let p2_screen = c.world_to_screen_coords(p2);
            assert_eq!(p2_screen, (560.0, 160.0));
            let p2_world = c.screen_to_world_coords(p2_screen);
            assert_eq!(p2_world, p2);
        }
    }
}
