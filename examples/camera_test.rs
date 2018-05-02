//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;

use ggez::conf;
use ggez::event;
use ggez::{Context, GameResult};
use ggez::graphics;
use ggez::graphics::{Point2, Vector2};
use ggez::timer;
use ggez::nalgebra as na;
use std::time::Duration;

extern crate ggez_goodies;
use ggez_goodies::camera::*;

struct MainState {
    camera: Camera,
    image: graphics::Image,
    image_location: graphics::Point2,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let camera = Camera::new(WINDOW_WIDTH, WINDOW_HEIGHT, CAMERA_WIDTH, CAMERA_HEIGHT);

        println!("Camera test instructions; WASD move the object, arrow keys move the camera.");
        println!(
            "The red dots are drawn on every integer point in the camera's coordinate \
             system."
        );
        let image = graphics::Image::solid(ctx, 5, graphics::Color::new(1.0, 0.0, 0.0, 1.0))?;
        graphics::set_background_color(ctx, ggez::graphics::Color::from((0, 0, 0, 0)));

        let state = MainState {
            camera: camera,
            image: image,
            image_location: graphics::Point2::new(0.0, 0.0),
        };
        Ok(state)
    }
}

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

const CAMERA_WIDTH: f32 = 40.0;
const CAMERA_HEIGHT: f32 = 30.0;

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let half_width = (CAMERA_WIDTH / 2.0) as i32;
        let half_height = (CAMERA_HEIGHT / 2.0) as i32;
        graphics::set_color(ctx, graphics::Color::from((255, 0, 0)))?;
        for y in -half_height..half_height {
            for x in -half_width..half_width {
                let frompt = Point2::new(x as f32, y as f32);
                let (px, py) = self.camera.world_to_screen_coords(frompt);
                let to = graphics::Point2::new(px as f32, py as f32);
                graphics::points(ctx, &[to], 1.0)?;
            }
        }
        self.image
            .draw_camera(&self.camera, ctx, self.image_location, 0.0)?;
        graphics::present(ctx);
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: event::Keycode,
        _keymod: event::Mod,
        _repeat: bool,
    ) {
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
            event::Keycode::Up => self.camera.move_by(Vector2::new(0.0, 0.1)),
            event::Keycode::Left => self.camera.move_by(Vector2::new(-0.1, 0.0)),
            event::Keycode::Down => self.camera.move_by(Vector2::new(0.0, -0.1)),
            event::Keycode::Right => self.camera.move_by(Vector2::new(0.1, 0.0)),
            _ => (),
        };
        println!(
            "Camera position is now {}, object position is {:?}",
            self.camera.location(),
            self.image_location
        );
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_setup.title = "Camera test".to_string();
    // c.window_width = WINDOW_WIDTH as u32;
    // c.window_height = WINDOW_HEIGHT as u32;
    let ctx = &mut Context::load_from_conf("camera_test", "test", c).unwrap();
    let game = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, game) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
