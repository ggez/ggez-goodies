//! A camera object for ggez.
//! Currently ggez has no actual global camera state to use,
//! so this is an attempt to give you coordinate transformations
//! into and out of the camera's coordinate space.
//!
//! It translates ggez's coordinate system with the origin
//! at the top-left and Y increasing downward to a coordinate system
//! with the origin at the center of the screen and Y increasing
//! upward.
//!
//! Because that makes sense, darn it.
//!
//! It also provides helper functions to do common camera movement tasks
//! like movement, rotation, and zooming.
//! It allows you to move the camera to world or screen-space coordinates
//! using easing function or physics-based smoothing.
//!
//! These should make it relatively easy to implement other game-specific
//! functions like pinning using whatever smoothing method you want.
//! 
//! A great source for how cameras can and should be used in 2d games is here:
//! http://www.gamasutra.com/blogs/ItayKeren/20150511/243083/Scroll_Back_The_Theory_and_Practice_of_Cameras_in_SideScrollers.php

use ggez;
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::timer;
use {Point2, Vector2, Matrix3, Isometry2, Translation2, Projective2};
use na::UnitComplex;
use na::{U1, U2};
use na;
use std::cmp;
use std::time::{Duration, Instant};
use std::f64::NAN;

type Matrix21 = na::Matrix2x1<f64>;

/// Determines how the the projection from the camera
/// to the screen will be made.
pub enum FitMode {
    /// Uniformly scales to the longer camera axis
    Fit,
    /// Uniformly scales to the shorter camera axis
    Fill,
    /// Non-uniformly scales to stretch both axes
    /// to match the respective camera axis scale
    Stretch
}

/// Represents a virtual camera in the game world.
/// The camera uses its own world coordinate system
/// that is distinct from the default ggez screen
/// coordinate system. These coordinates have their
/// origin at the center of the camera's original frame
/// and use a +Y-Up model, opposite of the screen
/// coordinates. 
pub struct Camera {
    screen_size: Vector2,
    view_size: Vector2,
    transform: Isometry2,
    projection: Projective2,
    action: Option<Box<CameraAction>>,
    start_scale: Vector2
}

fn partial_max<T: cmp::PartialOrd>(a: T, b: T) -> T {
    match a.partial_cmp(&b)
        .unwrap_or(cmp::Ordering::Less) {
        cmp::Ordering::Equal => a,
        cmp::Ordering::Greater => a,
        cmp::Ordering::Less => b,
    }
}

fn partial_min<T: cmp::PartialOrd>(a: T, b: T) -> T {
    match a.partial_cmp(&b)
        .unwrap_or(cmp::Ordering::Less) {
        cmp::Ordering::Equal => a,
        cmp::Ordering::Greater => b,
        cmp::Ordering::Less => a,
    }
}

impl Camera {
    /// Creates a new Camera given specified screen dimensions
    /// and camera view dimensions. It can often be useful to 
    /// define the camera world coordinate system in terms of
    /// your game's tile size, meaning that the view size would
    /// be the number of tiles on the screen at once.
    pub fn new(
        screen_width: u32, 
        screen_height: u32, 
        view_width: f64, 
        view_height: f64,
        stretch_mode: FitMode
    ) -> Self {
        let screen_size = Vector2::new(screen_width as f64, screen_height as f64);
        let view_size = Vector2::new(view_width as f64, view_height as f64);
        let pixels_per_unit = screen_size.component_div(&view_size);
        let pixels_per_unit = match stretch_mode {
            FitMode::Fit => {
                let u = partial_max(pixels_per_unit.x, pixels_per_unit.y);
                Vector2::new(u, u)
            },
            FitMode::Fill => {
                let u = partial_min(pixels_per_unit.x, pixels_per_unit.y);
                Vector2::new(u, u)
            },
            FitMode::Stretch => pixels_per_unit
        };
        let projection_matrix = Matrix3::new(
            pixels_per_unit.x,  0.0, screen_size.x / 2.0,
            0.0, -pixels_per_unit.y, screen_size.y / 2.0,
            0.0,  0.0, 1.0
        );
        let projection = Projective2::from_matrix_unchecked(projection_matrix);
        let transform = Isometry2::new(Vector2::new(0.0, 0.0), 0.0);
        Camera {
            screen_size,
            view_size,
            transform,
            projection,
            action: None,
            start_scale: pixels_per_unit
        }
    }

    /// Moves the camera by a world vector
    pub fn move_by_world(&mut self, by: Vector2) {
        self.transform
            .append_translation_mut(&Translation2::from_vector(by));
    }

    /// Moves the camera by a screen-space vector
    pub fn move_by_screen(&mut self, by: (f64, f64)) {
        let vec = self.projection.inverse() * Vector2::new(by.0, -by.1);
        self.move_by_world(vec);
    }

    /// Moves the camera to a world-space point
    pub fn move_to_world(&mut self, to: Point2) {
        self.transform.translation = Translation2::from_vector(to.coords);
    }

    /// Moves the camera to a screen-space point.
    pub fn move_to_screen(&mut self, to: (f64, f64)) {
        let pt = self.screen_to_world_coords(to);
        self.move_to_world(pt);
    }

    /// Eases between the camera's current position and a world-space Point
    /// using the selected Ease function over a duration
    pub fn move_towards_world_ease(&mut self, to: Point2, easer: Easer, duration: Duration, replace: bool) {
        let action = EaseAction::new(self.location(), to, easer, duration);
        if replace {
            self.action = Some(Box::new(action));
        } else {
            let mut replace = false;
            if let Some(ref mut a) = self.action {
                if let ActionType::Ease = a.get_type() {
                    replace = true;
                }
            } else {
                replace = true;
            }
            if replace {
                self.action = Some(Box::new(action));
            }
        }
    }

    /// Eases between the camera's current position and a screen-space Point
    /// using the selected Ease function over a duration
    pub fn move_towards_screen_ease(&mut self, to: (f64, f64), easer: Easer, duration: Duration, replace: bool) {
        let to = self.screen_to_world_coords(to);
        self.move_towards_world_ease(to, easer, duration, replace);
    }

    /// Moves to the specified world-space Point using physics-based smoothing
    pub fn move_towards_world_physics(&mut self, to: Point2, smoothing_mode: PhysicsSmoothingMode, mass: f64, replace: bool) {
        let action = PhysicsAction::new(to, smoothing_mode, mass);
        if replace {
            self.action = Some(Box::new(action));
        } else {
            if let Some(ref mut a) = self.action {
                if let ActionType::Physics = a.get_type() {
                    a.set_target(to);
                }
            } else {
                self.action = Some(Box::new(action));
            }
        }
    }

    /// Moves to the specified screen-space Point using physics-based smoothing
    pub fn move_towards_screen_physics(&mut self, to: (f64, f64), smoothing_mode: PhysicsSmoothingMode, mass: f64, replace: bool) {
        let to = self.screen_to_world_coords(to);
        self.move_towards_world_physics(to, smoothing_mode, mass, replace);
    }

    /// Rotates the camera about its center by by radians
    pub fn rotate_wrt_center_by(&mut self, by: f64) {
        self.transform
            .append_rotation_wrt_center_mut(&UnitComplex::new(by));
    }

    /// Rotates the camera about a world-space Point by by radians
    pub fn rotate_wrt_world_point_by(&mut self, point: Point2, by: f64) {
        self.transform
            .append_rotation_wrt_point_mut(&UnitComplex::new(by), &point);
    }

    /// Rotates the camera about a screen-space Point by by radians
    pub fn rotate_wrt_screen_point_by(&mut self, point: (f64, f64), by: f64) {
        let world_point = self.screen_to_world_coords(point);
        self.rotate_wrt_world_point_by(Point2::new(world_point.x, world_point.y), by);
    }

    /// Zooms the camera while keeping the center static 
    /// in the view (0.0-1.0 zooms out, > 1.0 zooms in)
    pub fn zoom_wrt_center_by(&mut self, by: f64) {
        self.projection.matrix_mut_unchecked().prepend_scaling_mut(by);
        self.view_size /= by;
    }

    /// Zooms the camera while keeping a world-space Point static
    /// in the view (0.0-1.0 zooms out, > 1.0 zooms in)
    pub fn zoom_wrt_world_point_by(&mut self, point: Point2, by: f64) {
        let scale_change = 1.0 - 1.0 / by;
        let dif = (point - self.location()) * scale_change;
        let translation = Translation2::new(dif.x, dif.y);
        self.projection.matrix_mut_unchecked().prepend_scaling_mut(by);
        self.transform.append_translation_mut(&translation);
        self.view_size /= by;
    }

    /// Zooms the camera while keeping a screen-space Point static
    /// in the view (0.0-1.0 zooms out, > 1.0 zooms in)
    pub fn zoom_wrt_screen_point_by(&mut self, point: (f64, f64), by: f64) {
        let world_point = self.screen_to_world_coords(point);
        self.zoom_wrt_world_point_by(world_point, by);
    }

    /// Translates a point in world-space to a point in
    /// screen-space.
    ///
    /// Does not do any clipping or anything, since it does
    /// not know how large the thing that might be drawn is;
    /// that's not its job.
    pub fn world_to_screen_coords(&self, from: Point2) -> (f64, f64) {
        let view = self.transform.inverse();
        let point = self.projection * view * from;
        (point.x, point.y)
    }


    /// Translates a point in screen-space to world-space
    pub fn screen_to_world_coords(&self, from: (f64, f64)) -> Point2 {
        let point = Point2::new(from.0, from.1);
        self.transform * self.projection.inverse() * point
    }

    /// Returns the camera's current location as a Point2
    pub fn location(&self) -> Point2 {
        Point2::from_coordinates(self.transform.translation.vector)
    }

    /// Translates a world-space point into screen-space and wraps it as a
    /// graphics::Point
    fn calculate_dest_point(&self, location: Point2) -> graphics::Point {
        let (sx, sy) = self.world_to_screen_coords(location);
        graphics::Point::new(sx as f32, sy as f32)
    }

    // Update ease and physics actions
    pub fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut action_status: Option<ActionStatus> = None;
        let location = self.location();
        if let Some(ref mut action) = self.action {
            action_status = Some(action.update(ctx, location));
        }
        if let Some(status) = action_status {
            match status {
                ActionStatus::Running(p) => {
                    self.move_to_world(p);
                },
                ActionStatus::Done(p) => {
                    self.move_to_world(p);
                    self.action = None;
                }
            }
        }
        Ok(())
    }

    pub fn debug_draw(
        &mut self, 
        ctx: &mut Context,
        screen_grid: bool,
        world_grid: bool
    ) -> GameResult<()> {
        graphics::set_color(ctx, graphics::Color::from((255, 0, 0)))?;
        if world_grid {
            let proj_mat = self.projection.to_homogeneous();
            let zoom = (
                length(&proj_mat.fixed_slice::<U2, U1>(0, 0).into_owned()) / self.start_scale.y,
                length(&proj_mat.fixed_slice::<U2, U1>(0, 1).into_owned()) / self.start_scale.x
            );
            let min_world_coords = self.location() - self.view_size;
            let min_world_coords = (min_world_coords.x as i64, min_world_coords.y as i64);
            let max_world_coords = self.location() + self.view_size;
            let max_world_coords = (max_world_coords.x as i64, max_world_coords.y as i64);
            for x in (min_world_coords.0)..(max_world_coords.0 + 1) {
                let mut scale = partial_max(1.0, zoom.1);
                scale *= if x % 10 == 0 { 3.0 } else { 1.0 };
                graphics::set_line_width(ctx, scale as f32);

                let points = [
                    self.calculate_dest_point(Point2::new(x as f64, self.view_size.y * 2.0)),
                    self.calculate_dest_point(Point2::new(x as f64, -self.view_size.y * 2.0))
                ];
                graphics::line(ctx, &points)?;
            }
            for y in (min_world_coords.1)..(max_world_coords.1 + 1) {
                let mut scale = partial_max(1.0, zoom.0);
                scale *= if y % 10 == 0 { 3.0 } else { 1.0 };
                graphics::set_line_width(ctx, scale as f32);

                let points = [
                    self.calculate_dest_point(Point2::new(-self.view_size.x * 2.0, y as f64)),
                    self.calculate_dest_point(Point2::new(self.view_size.x * 2.0, y as f64))
                ];
                graphics::line(ctx, &points)?;
            }
        }
        graphics::set_color(ctx, graphics::Color::from((100, 120, 255)))?;
        if screen_grid {
            let scaling = partial_min(self.start_scale.x, self.start_scale.y) as u64 * 5;
            for x in 0..self.screen_size.x as u64 {
                if x % scaling == 0 {
                    let px = x as f32;
                    let scale = if x % (scaling * 2) == 0 { 3.0 } else { 1.0 };
                    graphics::set_line_width(ctx, scale as f32);

                    let points = [
                        graphics::Point::new(px, 0.0),
                        graphics::Point::new(px, self.screen_size.y as f32)
                    ];
                    graphics::line(ctx, &points)?;
                }
            }
            for y in 0..self.screen_size.y as u64 {
                if y % scaling == 0 {
                    let py = y as f32;
                    let scale = if y % (scaling * 2) == 0 { 3.0 } else { 1.0 };
                    graphics::set_line_width(ctx, scale as f32);

                    let points = [
                        graphics::Point::new(0.0, py),
                        graphics::Point::new(self.screen_size.x as f32, py)
                    ];
                    graphics::line(ctx, &points)?;
                }
            }
        }
        Ok(())
    }
}

fn length(vec: &Matrix21) -> f64 {
    vec.dot(&vec.normalize())
}

/// Allows a `Drawable` to be drawn from the Camera's perspective.
pub trait CameraDraw
    where Self: graphics::Drawable
{
    fn draw_ex_camera(&self,
                      camera: &Camera,
                      ctx: &mut ggez::Context,
                      p: ggez::graphics::DrawParam
    ) -> GameResult<()> {
        let dest = Point2::new(p.dest.x as f64, p.dest.y as f64);
        let dest = camera.calculate_dest_point(dest);
        let mut my_p = p;
        my_p.dest = dest;
        my_p.rotation = my_p.rotation + camera.transform.rotation.angle() as f32;
        let proj_mat = camera.projection.to_homogeneous();
        let scale = (
            length(&proj_mat.fixed_slice::<U2, U1>(0, 0).into_owned()) / camera.start_scale.y, 
            length(&proj_mat.fixed_slice::<U2, U1>(0, 1).into_owned()) / camera.start_scale.x
        );
        my_p.scale = graphics::Point::new(scale.0 as f32 * my_p.scale.x, scale.1 as f32 * my_p.scale.y);
        self.draw_ex(ctx, my_p)
    }

    fn draw_camera(&self,
                   camera: &Camera,
                   ctx: &mut ggez::Context,
                   dest: ggez::graphics::Point,
                   rotation: f32
    ) -> GameResult<()> {
        let dest = Point2::new(dest.x as f64, dest.y as f64);
        let dest = camera.calculate_dest_point(dest);
        let rotation = rotation + camera.transform.rotation.angle() as f32;
        let proj_mat = camera.projection.to_homogeneous();
        let scale = (
            length(&proj_mat.fixed_slice::<U2, U1>(0, 0).into_owned()) / camera.start_scale.y, 
            length(&proj_mat.fixed_slice::<U2, U1>(0, 1).into_owned()) / camera.start_scale.x
        );
        let scale = graphics::Point::new(scale.0 as f32, scale.1 as f32);
        self.draw_ex(
            ctx,
            graphics::DrawParam{
                dest,
                rotation,
                scale,
                .. Default::default()
            }
        )
    }
}


impl<T> CameraDraw for T where T: graphics::Drawable {}

trait CameraAction {
    fn update(&mut self, ctx: &Context, position: Point2) -> ActionStatus;
    fn set_target(&mut self, target: Point2);
    fn get_type(&self) -> ActionType;
}

/// The type of smoothing to use for physics-smoothed camera movements
pub enum PhysicsSmoothingMode {
    /// Applies a constant force in the direction of the target
    ConstantForce(f64),
    /// Applies a force proportional to the distance between the target and
    /// the current position (difference * p)
    ProportionalForce(f64),
    /// Uses a PID controller to determine the force to apply based on the
    /// a proportion of error, the integral of errors, and the derivative
    /// of the error. See `PIDConfiguration` for more information.
    PIDControlled(PIDConfiguration)
}

enum ActionType {
    Ease,
    Physics
}

struct PhysicsAction {
    controller: Box<PhysicsController>,
    velocity: Vector2,
    mass: f64,
    target: Point2
}

impl PhysicsAction {
    pub fn new(target: Point2, smoothing_mode: PhysicsSmoothingMode, mass: f64) -> Self {
        let controller: Box<PhysicsController> = match smoothing_mode {
            PhysicsSmoothingMode::PIDControlled(conf) => {
                Box::new(PIDController2D::new(Point2::origin(), conf))
            },
            PhysicsSmoothingMode::ConstantForce(force) => {
                Box::new(PhysicsConstantForceController::new(Point2::origin(), force))
            },
            PhysicsSmoothingMode::ProportionalForce(p) => {
                Box::new(PIDController2D::new(
                    target,
                    PIDConfiguration::new(p, 0.0, 0.0)
                ))
            }
        };
        PhysicsAction {
            controller,
            mass,
            velocity: Vector2::new(0.0, 0.0),
            target
        }
    }
}

static REST_THRESHOLD: f64 = 0.0001;

impl CameraAction for PhysicsAction {
    fn update(&mut self, ctx: &Context, position: Point2) -> ActionStatus {
        let val = position - self.target;
        let force = self.controller.update(val, timer::duration_to_f64(timer::get_delta(ctx))) / self.mass;
        self.velocity = self.velocity + force;
        let velocity_at_rest = self.velocity.x.abs() < REST_THRESHOLD && self.velocity.y.abs() < REST_THRESHOLD;
        let impulse_at_rest = force.x.abs() < REST_THRESHOLD && force.y.abs() < REST_THRESHOLD;
        if velocity_at_rest && impulse_at_rest {
            ActionStatus::Done(position)
        } else {
            ActionStatus::Running(position + self.velocity)
        }
    }
    fn set_target(&mut self, target: Point2) {
        self.controller.set_target(target);
    }
    fn get_type(&self) -> ActionType {
        ActionType::Physics
    }
}

trait PhysicsController {
    fn set_target(&mut self, target: Point2);
    fn target(&self) -> Point2;
    fn update(&mut self, value: Vector2, dt: f64) -> Vector2;
}

struct PhysicsConstantForceController {
    target: Point2,
    impulse_mag: f64
}

impl PhysicsConstantForceController {
    pub fn new(target: Point2, impulse_mag: f64) -> Self {
        PhysicsConstantForceController {
            target,
            impulse_mag
        }
    }
}

impl PhysicsController for PhysicsConstantForceController {
    fn set_target(&mut self, target: Point2) {
        self.target = target;
    }

    fn target(&self) -> Point2 {
        self.target
    }
    fn update(&mut self, value: Vector2, _dt: f64) -> Vector2 {
        let err = self.target - value;
        let err = Vector2::new(err.x, err.y);
        if length(&err) > self.impulse_mag {
            err.normalize() * self.impulse_mag
        } else {
            err
        }
    }
}

struct PIDController2D {
    x: PIDController,
    y: PIDController
}

impl PIDController2D {
    pub fn new(target: Point2, config: PIDConfiguration) -> Self {
        PIDController2D {
            x: PIDController::new(target.x, config),
            y: PIDController::new(target.y, config)
        }
    }
}

impl PhysicsController for PIDController2D {
    fn set_target(&mut self, target: Point2) {
        self.x.set_target(target.x);
        self.y.set_target(target.y);
    }
    fn target(&self) -> Point2 {
        Point2::new(self.x.target(), self.y.target())
    }
    fn update(&mut self, value: Vector2, dt: f64) -> Vector2 {
        Vector2::new(self.x.update(value.x, dt), self.y.update(value.y, dt))
    }
}

/// The configuration necessary for a PID controller to handle smoothing
/// of physics-smoothed actions. 
///
/// For more information about what a PID
/// controller is, how it works, and how to use one, see:
///
/// https://www.youtube.com/watch?v=0vqWyramGy8
///
/// https://www.youtube.com/watch?v=fusr9eTceEo
///
/// https://www.youtube.com/watch?v=qKy98Cbcltw
#[derive(Clone, Copy)]
pub struct PIDConfiguration {
    p: f64,
    i: f64,
    d: f64,
}

impl PIDConfiguration {
    pub fn new(p: f64, i: f64, d: f64) -> Self {
        PIDConfiguration {
            p,
            i,
            d
        }
    }
}

struct PIDController {
    config: PIDConfiguration,
    target: f64,
    err_integral: f64,
    prev_value: f64,
    prev_err: f64
}

impl PIDController {
    pub fn new(target: f64, config: PIDConfiguration) -> Self {
        PIDController {
            config,
            target,
            err_integral: 0.0,
            prev_value: NAN,
            prev_err: NAN
        }
    }
    pub fn set_target(&mut self, target: f64) {
        self.target = target;
    }
    pub fn target(&self) -> f64 {
        self.target
    }
    pub fn update(&mut self, value: f64, dt: f64) -> f64 {
        let err = self.target - value;

        let p_term = self.config.p * err;
        self.err_integral += self.config.i * err * dt;
        let i_term = self.err_integral;
        let d_term = if self.prev_err.is_nan() || self.prev_value.is_nan() {
            0.0
        } else {
            self.config.d * (self.prev_value - value) / dt
        };

        self.prev_err = err;
        self.prev_value = value;

        p_term + i_term + d_term
    }
}

struct EaseAction {
    start_point: Point2,
    change_vec: Vector2,
    easer: Easer,
    start_time: Instant,
    duration: f64
}

/// The function signature required for an easing function. This works excellently
/// with [ezing](https://github.com/michaelfairley/ezing), and we highly recommend
/// using it with that library, as shown in the example.
/// 
/// Expects input ranging from `0.0` to `1.0` and should map `0.0` to `0.0`
/// and `1.0` to `1.0`.
pub type Easer = fn(f32) -> f32;

impl EaseAction {
    pub fn new(
        start_point: Point2,
        end_point: Point2,
        easer: Easer,
        duration: Duration
    ) -> Self {
        let change_vec = end_point - start_point;
        let duration = timer::duration_to_f64(duration);
        EaseAction {
            start_point,
            change_vec,
            easer,
            start_time: Instant::now(),
            duration
        }
    }
}
impl CameraAction for EaseAction {
    fn update(&mut self, _ctx: &Context, _position: Point2) -> ActionStatus {
        let t = timer::duration_to_f64(self.start_time.elapsed()) / self.duration;
        if t >= 1.0 {
            ActionStatus::Done(self.start_point + self.change_vec)
        } else {
            ActionStatus::Running(self.start_point + self.change_vec * (self.easer)(t as f32) as f64)
        }
    }
    fn set_target(&mut self, target: Point2) {
        self.change_vec = target - self.start_point;
    }
    fn get_type(&self) -> ActionType {
        ActionType::Ease
    }
}

enum ActionStatus {
    Running(Point2),
    Done(Point2)
}


#[cfg(test)]
mod tests {
    use Vector2;
    use super::*;

    #[test]
    fn test_coord_round_trip() {
        let mut c = Camera::new(640, 480, 40.0, 30.0, FitMode::Fit);
        let p1 = (200.0, 300.0);
        let p2 = Point2::new(20.0, 10.0);
        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Point2::new(-7.5, -3.75));
            let p1_screen = c.world_to_screen_coords(p1_world);
            assert_eq!(p1, p1_screen);
        }


        {
            let p2_screen = c.world_to_screen_coords(p2);
            assert_eq!(p2_screen, (640.0, 80.0));
            let p2_world = c.screen_to_world_coords(p2_screen);
            assert_eq!(p2_world, p2);
        }
    }

    #[test]
    fn test_move_to_round_trip() {
        let mut c = Camera::new(640, 480, 40.0, 30.0, FitMode::Fit);
        let p1 = (200.0, 300.0);
        let p2 = Point2::new(20.0, 10.0);

        c.move_to_world(Point2::new(5.0, 5.0));
        {
            println!("camera pos: {}", c.location());
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
        
        c.move_to_screen((240.0, 320.0));
        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Point2::new(-7.5, -3.75));
            let p1_screen = c.world_to_screen_coords(p1_world);
            assert_eq!(p1, p1_screen);
        }
        {
            let p2_screen = c.world_to_screen_coords(p2);
            assert_eq!(p2_screen, (640.0, 80.0));
            let p2_world = c.screen_to_world_coords(p2_screen);
            assert_eq!(p2_world, p2);
        }
    }

    #[test]
    fn test_move_by_round_trip() {
        let mut c = Camera::new(640, 480, 40.0, 30.0, FitMode::Fit);
        let p1 = (200.0, 300.0);
        let p2 = Point2::new(20.0, 10.0);

        c.move_by_world(Vector2::new(5.0, 5.0));
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

        c.move_by_screen((-80.0, -80.0));
        {
            let p1_world = c.screen_to_world_coords(p1);
            assert_eq!(p1_world, Point2::new(-7.5, -3.75));
            let p1_screen = c.world_to_screen_coords(p1_world);
            assert_eq!(p1, p1_screen);
        }
        {
            let p2_screen = c.world_to_screen_coords(p2);
            assert_eq!(p2_screen, (640.0, 80.0));
            let p2_world = c.screen_to_world_coords(p2_screen);
            assert_eq!(p2_world, p2);
        }
    }
}
