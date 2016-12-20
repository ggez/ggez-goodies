//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;

use std::path;

use ggez::audio;
use ggez::conf;
use ggez::event;
use ggez::game::{Game, GameState};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::graphics::Drawable;
use ggez::timer;
use std::time::Duration;
use std::ops::{Add, AddAssign, Sub};


extern crate nalgebra as na;
type Vector2 = na::Vector2<f64>;

extern crate ggez_goodies;
use ggez_goodies::camera::*;

struct MainState {
    camera: Camera,
    image: graphics::Image,
}

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

impl GameState for MainState {
    fn load(ctx: &mut Context, conf: &conf::Conf) -> GameResult<Self> {
        let camera = Camera::new(WINDOW_WIDTH, WINDOW_HEIGHT, 40.0, 30.0);
        let image = graphics::Image::new(ctx, "tile.png")?;
        let state = MainState {
            camera: camera,
            image: image,
        };
        graphics::set_background_color(ctx, ggez::graphics::Color::RGBA(0, 0, 0, 0));
        Ok(state)
    }
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let dest_rect = ggez::graphics::Rect::new(0, 0, 0, 0);
        self.image.draw(ctx, None, Some(dest_rect))?;
        self.image
            .draw_camera(&self.camera,
                         na::Vector2::new(0.0, 0.0),
                         ctx,
                         None,
                         (64, 64))?;

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
                event::Keycode::W | event::Keycode::Up => {
                    self.camera.move_by(Vector2::new(0.0, 1.0))
                }
                event::Keycode::A | event::Keycode::Left => {
                    self.camera.move_by(Vector2::new(-1.0, 0.0))
                }
                event::Keycode::S | event::Keycode::Down => {
                    self.camera.move_by(Vector2::new(0.0, -1.0))
                }
                event::Keycode::D | event::Keycode::Right => {
                    self.camera.move_by(Vector2::new(1.0, 0.))
                }
                _ => (),
            };
            println!("Camera position is now {}", self.camera.location());
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
