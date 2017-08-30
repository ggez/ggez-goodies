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
//! Does easing, but no pinning or other advanced camera techniques.
//! These should be relatively easy to implement given the built in
//! easing and movement functions.
//! 
//! A great source for how such things work is this:
//! http://www.gamasutra.com/blogs/ItayKeren/20150511/243083/Scroll_Back_The_Theory_and_Practice_of_Cameras_in_SideScrollers.php

// TODO: Debug functions to draw world and camera grid!

use ggez;
use ggez::GameResult;
use ggez::graphics;
use {Point2, Vector2, Matrix3, Similarity2, Translation2, Projective2};
use na::UnitComplex;
use std::cmp;

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
        // Similarities only support uniform scaling, in case scale factor is not uniform, use the smaller one
        let units_per_pixel = match units_per_pixel.x.partial_cmp(&units_per_pixel.y).unwrap_or(cmp::Ordering::Less) {
            cmp::Ordering::Equal => units_per_pixel.x,
            cmp::Ordering::Greater => units_per_pixel.y,
            cmp::Ordering::Less => units_per_pixel.x
        };
        let transform = Similarity2::new(Vector2::new(0.0, 0.0), 0.0, units_per_pixel);
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

    // Move the camera by the vector based on the global axes
    pub fn move_by_global(&mut self, by: Vector2) {
        self.transform.append_translation_mut(&Translation2::from_vector(by));
    }

    // Move the camera by the vector based on the local camera transformation axes
    pub fn move_by_local(&mut self, by: Vector2) {
        let vec = self.transform.isometry.rotation * by;
        self.move_by_global(vec);
    }

    // Interpolate with a cubic ease-in-out algorithm from start to start + change where the x axis is from 0.0 - 1.0 represented by t
    pub fn ease_in_out_cub(start: f64, change: f64, t: f64) -> f64 {
        let t = t * 2.0;
        if t < 1.0 {
            change/2.0*t*t*t + start
        } else {
            let t = t - 2.0;
            change/2.0*(t*t*t + 2.0) + start
        }
    }

    // Interpolate with a quadratic ease-in-out algorithm from start to start + change where the x axis is from 0.0 - 1.0 represented by t
    pub fn ease_in_out_quad(start: f64, change: f64, t: f64) -> f64 {
        let t = t * 2.0;
        if t < 1.0 {
            change/2.0*t*t + start
        } else {
            let t = t - 1.0;
            -change/2.0 * (t*(t-2.0) - 1.0) + start
        }
    }

    // Linearly interpolate between the camera's current position and a world-space Point by t where t is 0.0 - 1.0
    pub fn move_towards_world_lerp(&mut self, to: Point2, t: f64) {
        let dif = (to - self.location()) * t;
        self.transform.append_translation_mut(&Translation2::new(dif.x, dif.y));
    }
    // Linearly interpolate between the camera's current position and a screen-space Point by t where t is 0.0 - 1.0
    pub fn move_towards_screen_lerp(&mut self, to: (f64, f64), t: f64) {
        let vec = (self.screen_to_world_coords(to) - self.location().coords) * t;
        self.transform.append_translation_mut(&Translation2::from_vector(vec.coords));
    }

    // Interpolate between the camera's current position and a world-space Point with a cubic ease-in-out curve
    // where progress of the interpolation is represented by t from 0.0 - 1.0
    pub fn move_towards_world_ease_cub(&mut self, to: Point2, t: f64) {
        let dif = to - self.location();
        let mut vec = Vector2::new(dif.x, dif.y);
        vec.x = Camera::ease_in_out_cub(0.0, vec.x, t);
        vec.y = Camera::ease_in_out_cub(0.0, vec.y, t);
        self.transform.append_translation_mut(&Translation2::from_vector(vec));
    }
    // Interpolate between the camera's current position and a screen-space Point with a cubic ease-in-out curve
    // where progress of the interpolation is represented by t from 0.0 - 1.0
    pub fn move_towards_screen_ease_cub(&mut self, to: (f64, f64), t: f64) {
        let global = self.screen_to_world_coords(to);
        self.move_towards_world_ease_cub(Point2::new(global.x, global.y), t);
    }

    // Interpolate between the camera's current position and a world-space Point with a quadratic ease-in-out curve
    // where progress of the interpolation is represented by t from 0.0 - 1.0
    pub fn move_towards_world_ease_quad(&mut self, to: Point2, t: f64) {
        let mut dif = to - self.location();
        dif.x = Camera::ease_in_out_quad(0.0, dif.x, t);
        dif.y = Camera::ease_in_out_quad(0.0, dif.y, t);
        self.transform.append_translation_mut(&Translation2::from_vector(dif));
    }
    // Interpolate between the camera's current position and a screen-space Point with a quadratic ease-in-out curve
    // where progress of the interpolation is represented by t from 0.0 - 1.0
    pub fn move_towards_screen_ease_quad(&mut self, to: (f64, f64), t: f64) {
        let global = self.screen_to_world_coords(to);
        self.move_towards_world_ease_quad(Point2::new(global.x, global.y), t);
    }

    // Interpolate between the camera's current position and a world-space Point with a given ease function where
    // the easing function has signature fn(start: f64, change: f64, t: f64) -> f64
    // where progress of the interpolation is represented by t from 0.0 - 1.0
    pub fn move_towards_world_ease(&mut self, to: Point2, t: f64, ease_function: &Fn(f64, f64, f64) -> f64) {
        let dif = to - self.location();
        let mut vec = Vector2::new(dif.x, dif.y);
        vec.x = ease_function(0.0, vec.x, t);
        vec.y = ease_function(0.0, vec.y, t);
        self.transform.append_translation_mut(&Translation2::from_vector(vec));
    }
    // Interpolate between the camera's current position and a screen-space Point with a given ease function where
    // the easing function has signature fn(start: f64, change: f64, t: f64) -> f64
    // where progress of the interpolation is represented by t from 0.0 - 1.0
    pub fn move_towards_screen_ease(&mut self, to: (f64, f64), t: f64, ease_function: &Fn(f64, f64, f64) -> f64) {
        let global = self.screen_to_world_coords(to);
        self.move_towards_world_ease(Point2::new(global.x, global.y), t, ease_function);
    }

    // Rotate the camera about its center by by radians
    pub fn rotate_wrt_center_by(&mut self, by: f64) {
        self.transform.append_rotation_wrt_center_mut(&UnitComplex::new(by));
    }

    // Rotate the camera about a world-space Point by by radians
    pub fn rotate_wrt_world_point_by(&mut self, point: Point2, by: f64) {
        self.transform.append_rotation_wrt_point_mut(&UnitComplex::new(by), &point);
    }

    // Rotate the camera about a screen-space Point by by radians
    pub fn rotate_wrt_screen_point_by(&mut self, point: (f64, f64), by: f64) {
        let world_point = self.screen_to_world_coords(point);
        self.rotate_wrt_world_point_by(Point2::new(world_point.x, world_point.y), by);
    }
    pub fn move_to(&mut self, to: Vector2) {
        self.transform.isometry.translation = Translation2::from_vector(to);
    }

    // Zoom the camera while keeping the center static  in the view (0.0-1.0 zooms out, > 1.0 zooms in)
    pub fn zoom_wrt_center_by(&mut self, by: f64) {
        self.zoom *= by;
        self.transform.prepend_scaling_mut(1.0 / by);
    }

    // Zoom the camera while a world-space Point static in the view (0.0-1.0 zooms out, > 1.0 zooms in)
    pub fn zoom_wrt_world_point_by(&mut self, point: Point2, by: f64) {
        self.zoom *= by;
        let by = 1.0 / by;
        let scale_change = 1.0 - by;
        let dif = (point - self.location()) * scale_change;
        let translation = Translation2::new(dif.x, dif.y);
        self.transform.prepend_scaling_mut(by);
        self.transform.append_translation_mut(&translation);
    }

    // Zoom the camera while a screen-space Point static in the view (0.0-1.0 zooms out, > 1.0 zooms in)
    pub fn zoom_wrt_screen_point_by(&mut self, point: (f64, f64), by: f64) {
        let world_point = self.screen_to_world_coords(point);
        self.zoom_wrt_world_point_by(world_point, by);
    }

    // Translates a point in world-space to a point in
    // screen-space.
    //
    // Does not do any clipping or anything, since it does
    // not know how large the thing that might be drawn is;
    // that's not its job.
    pub fn world_to_screen_coords(&self, from: Point2) -> (f64, f64) {
        let camera_transform = self.transform.inverse();
        let point_camera = camera_transform * from;
        let point_screen = self.screen_transform * point_camera;
        (point_screen.x, point_screen.y)
    }


    // Translates a point in screen-space to world-space
    pub fn screen_to_world_coords(&self, from: (f64, f64)) -> Point2 {
        let point = Point2::new(from.0 as f64, from.1 as f64);
        let point_world = self.screen_transform.inverse() * point;
        self.transform * point_world
    }

    // Returns the camera's current location as a vector with origin at the world-space origin
    pub fn location(&self) -> Point2 {
        Point2::from_coordinates(self.transform.isometry.translation.vector)
    }

    // Translates a world-space point into screen-space and wraps it as a
    // graphics::Point
    fn calculate_dest_point(&self, location: Point2) -> graphics::Point {
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
        let dest = Point2::new(p.dest.x as f64, p.dest.y as f64);
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
        let dest = Point2::new(dest.x as f64, dest.y as f64);
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
            assert_eq!(p1_world, Point2::new(-7.5, -3.75));
            let p1_screen = c.world_to_screen_coords(p1_world);
            assert_eq!(p1, p1_screen);
        }


        let p2 = Point2::new(20.0, 10.0);
        {
            let p2_screen = c.world_to_screen_coords(p2);
            assert_eq!(p2_screen, (640.0, 80.0));
            let p2_world = c.screen_to_world_coords(p2_screen);
            assert_eq!(p2_world, p2);
        }

        c.move_to(Vector2::new(5.0, 5.0));

        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Point2::new(-2.5, 1.25));
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
