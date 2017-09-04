//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;
extern crate ezing;

use ggez::conf;
use ggez::event;
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::timer;
use std::time::Duration;


extern crate nalgebra as na;

extern crate ggez_goodies;
use ggez_goodies::camera::*;
use ggez_goodies::{Vector2, Point2};

struct MainState {
    camera: Camera,
    image: graphics::Image,
    image_location: graphics::Point,
    mouse_pos: (f64, f64)
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let camera = Camera::new(WINDOW_WIDTH, WINDOW_HEIGHT, CAMERA_WIDTH, CAMERA_HEIGHT, FitMode::Stretch);

        println!("Camera test instructions:");
        println!("WASD to move the player");
        println!("IJKL to move the camera with respect to global axes");
        println!("Arrow keys to move the camera with respect to local camera axes\n");
        println!("QE to rotate the camera with respect to its center");
        println!("RT to rotate the camera with respect to the player center");
        println!("ZX to zoom the camera with respect to the camera center");
        println!("CV to zoom the camera with respect to the player center\n");
        println!("Left click to move the camera to the mouse cursor with cubic-in-out easing");
        println!("Right click to move the camera to the mouse cursor with elastic-out easing");
        println!("Middle click to move the camera to the mouse cursor with linear easing\n");
        println!("Space to move the camera to center on the player position with physics smoothing\n");
        println!("The red lines are drawn on every integer point in the world coordinate \
                  system.");
        println!("The blue lines represent the screen coordinate system and are drawn based \
                  on the integer world points contained in the camera's original field of view");
        let image = graphics::Image::new(ctx, "/player.png")?;
        graphics::set_background_color(ctx, ggez::graphics::Color::from((0, 0, 0, 0)));
        let state = MainState {
            camera,
            image,
            image_location: graphics::Point::zero(),
            mouse_pos: (0., 0.)
        };
        Ok(state)
    }
}

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 960;

const CAMERA_WIDTH: f64 = 40.0;
const CAMERA_HEIGHT: f64 = 30.0;

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        self.camera.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        use graphics::*;
        clear(ctx);

        set_color(ctx, Color::from((255, 0, 0)))?;
        self.image.draw_camera(&self.camera, ctx, self.image_location, 0.0)?;
        self.camera.debug_draw(ctx, true, true)?;

        present(ctx);
        timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }

    fn mouse_motion_event(&mut self, _state: event::MouseState, x: i32, y: i32, _xrel: i32, _yrel: i32) {
        self.mouse_pos = (x as f64, y as f64);
    }

    fn key_down_event(&mut self, keycode: event::Keycode, _keymod: event::Mod, _repeat: bool) {
        match keycode {
            event::Keycode::W => {
                self.image_location.y += 0.1;
            }
            event::Keycode::A => {
                self.image_location.x -= 0.1;
            }
            event::Keycode::S => {
                self.image_location.y -= 0.1;
            }
            event::Keycode::D => {
                self.image_location.x += 0.1;
            }
            event::Keycode::I => self.camera.move_by_world(Vector2::new(0.0, 0.1)),
            event::Keycode::J => self.camera.move_by_world(Vector2::new(-0.1, 0.0)),
            event::Keycode::K => self.camera.move_by_world(Vector2::new(0.0, -0.1)),
            event::Keycode::L => self.camera.move_by_world(Vector2::new(0.1, 0.0)),
            event::Keycode::Up => self.camera.move_by_screen((0.0, 2.0)),
            event::Keycode::Left => self.camera.move_by_screen((-2.0, 0.0)),
            event::Keycode::Down => self.camera.move_by_screen((0.0, -2.0)),
            event::Keycode::Right => self.camera.move_by_screen((2.0, 0.0)),
            event::Keycode::Q => self.camera.rotate_wrt_center_by(-0.01),
            event::Keycode::E => self.camera.rotate_wrt_center_by(0.01),
            event::Keycode::R => self.camera.rotate_wrt_world_point_by(Point2::new(self.image_location.x as f64, self.image_location.y as f64), -0.01),
            event::Keycode::T => self.camera.rotate_wrt_world_point_by(Point2::new(self.image_location.x as f64, self.image_location.y as f64), 0.01),
            event::Keycode::Z => self.camera.zoom_wrt_center_by(1.25),
            event::Keycode::X => self.camera.zoom_wrt_center_by(0.8),
            event::Keycode::C => self.camera.zoom_wrt_world_point_by(Point2::new(self.image_location.x as f64, self.image_location.y as f64), 1.25),
            event::Keycode::V => self.camera.zoom_wrt_world_point_by(Point2::new(self.image_location.x as f64, self.image_location.y as f64), 0.8),
            event::Keycode::Space => self.camera.move_to_world_physics(
                Point2::new(self.image_location.x as f64, self.image_location.y as f64),
                PhysicsSmoothingMode::PIDControlled(PIDConfiguration::new(0.225, 0.0, 0.06)),
                40.0,
                false
            ),
            _ => (),
        };
    }

    fn mouse_button_down_event(&mut self, btn: event::MouseButton, x: i32, y: i32) {
        match btn {
            event::MouseButton::Left => self.camera.move_to_screen_ease((x as f64, y as f64), ezing::cubic_inout, Duration::from_millis(1000), false),
            event::MouseButton::Middle => self.camera.move_to_screen_ease((x as f64, y as f64), lerp, Duration::from_millis(1000), false),
            event::MouseButton::Right => self.camera.move_to_screen_ease((x as f64, y as f64), ezing::elastic_out, Duration::from_millis(2500), false),
            _ => ()
        }
    }
}

fn lerp(t: f32) -> f32 {t}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Camera test".to_string();
    c.window_width = WINDOW_WIDTH as u32;
    c.window_height = WINDOW_HEIGHT as u32;
    let ctx = &mut Context::load_from_conf("camera_test", "test", c).unwrap();
    let game = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, game) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
