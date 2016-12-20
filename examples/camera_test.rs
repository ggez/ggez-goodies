//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;

use ggez::conf;
use ggez::event;
use ggez::game::{Game, GameState};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::graphics::Drawable;
use ggez::timer;
use std::time::Duration;


extern crate nalgebra as na;
type Vector2 = na::Vector2<f64>;

extern crate ggez_goodies;
use ggez_goodies::camera::*;

struct MainState {
    camera: Camera,
    image: graphics::Image,
    image_location: Vector2,
}

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

const CAMERA_WIDTH: f64 = 40.0;
const CAMERA_HEIGHT: f64 = 30.0;

impl GameState for MainState {
    fn load(ctx: &mut Context, _conf: &conf::Conf) -> GameResult<Self> {
        let camera = Camera::new(WINDOW_WIDTH, WINDOW_HEIGHT, CAMERA_WIDTH, CAMERA_HEIGHT);

        println!("Camera test instructions; WASD move the object, arrow keys move the camera.");
        println!("The red dots are drawn on every integer point in the camera's coordinate \
                  system.");
        let image = graphics::Image::new(ctx, "tile.png")?;
        graphics::set_background_color(ctx, ggez::graphics::Color::RGBA(0, 0, 0, 0));
        let state = MainState {
            camera: camera,
            image: image,
            image_location: na::zero(),
        };
        Ok(state)
    }
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let dest_rect = ggez::graphics::Rect::new(0, 0, 0, 0);

        let half_width = (CAMERA_WIDTH / 2.0) as i32;
        let half_height = (CAMERA_HEIGHT / 2.0) as i32;
        graphics::set_color(ctx, graphics::Color::RGB(255, 0, 0));
        for y in -half_height..half_height {
            for x in -half_width..half_width {
                let fromvec = Vector2::new(x as f64, y as f64);
                let (px, py) = self.camera.world_to_screen_coords(fromvec);
                let to = graphics::Point::new(px, py);
                graphics::point(ctx, to)?;
            }
        }
        self.image
            .draw_camera(&self.camera, self.image_location, ctx, None, (64, 64))?;
        graphics::present(ctx);
        timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }


    fn key_down_event(&mut self,
                      keycode: Option<event::Keycode>,
                      _keymod: event::Mod,
                      _repeat: bool) {
        if let Some(key) = keycode {
            match key {
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
            println!("Camera position is now {}, object position is {}",
                     self.camera.location(),
                     self.image_location);
        }

    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Shiny particles".to_string();
    c.window_width = WINDOW_WIDTH as u32;
    c.window_height = WINDOW_HEIGHT as u32;
    let game: GameResult<Game<MainState>> = Game::new("camera_test", c);
    match game {
        Err(e) => {
            println!("Could not load game!");
            println!("Error: {:?}", e);
        }
        Ok(mut game) => {
            let result = game.run();
            if let Err(e) = result {
                println!("Error encountered running game: {:?}", e);
            } else {
                println!("Game exited cleanly.");
            }
        }
    }
}
