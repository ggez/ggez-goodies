//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;

use std::path;

use ggez::audio;
use ggez::conf;
use ggez::game::{Game, GameState};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::timer;
use std::time::Duration;
use std::ops::{Add, AddAssign, Sub};

extern crate ggez_goodies;
use ggez_goodies::particle::*;

struct MainState {
    particles: ParticleSystem,
}

const window_width: i32 = 640;
const window_height: i32 = 480;

impl GameState for MainState {
    fn load(ctx: &mut Context, conf: &conf::Conf) -> GameResult<Self> {
        let system = ParticleSystemBuilder::new()
            .count(50)
            .lifetime(2.0)
            .build();
        let state = MainState { particles: system };
        graphics::set_background_color(ctx, ggez::graphics::Color::RGBA(0, 0, 0, 0));
        Ok(state)
    }
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()> {
        let seconds = timer::duration_to_f64(dt);
        self.particles.emit();
        self.particles.update(seconds);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let dest_rect = ggez::graphics::Rect::new(window_width / 2, window_height / 2, 0, 0);
        graphics::draw(ctx, &self.particles, None, Some(dest_rect))?;

        graphics::present(ctx);
        timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Shiny particles".to_string();
    c.window_width = window_width as u32;
    c.window_height = window_height as u32;
    let game: GameResult<Game<MainState>> = Game::new("shinyparticles", c);
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
