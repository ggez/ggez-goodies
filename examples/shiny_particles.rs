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


extern crate nalgebra as na;
type Vector2 = na::Vector2<f64>;

extern crate ggez_goodies;
use ggez_goodies::particle::*;

struct MainState {
    particles: ParticleSystem,
}

const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;

impl GameState for MainState {
    fn load(ctx: &mut Context, conf: &conf::Conf) -> GameResult<Self> {
        let system = ParticleSystemBuilder::new(ctx)
            .count(50000)
            .acceleration(Vector2::new(0.0, 50.0))
            .start_max_age(15.0)
            .start_size_range(2.0, 15.0)
            .start_color_range(graphics::Color::RGB(0, 0, 0),
                               graphics::Color::RGB(255, 255, 255))
            .start_velocity_range(
                Vector2::new(-50.0, -200.0),
                Vector2::new( 50.0, 0.0)
            )
            .start_rotation_range(-10.0, 10.0)
            .emission_rate(1000.0)
            
            .build();
        let state = MainState { particles: system };
        graphics::set_background_color(ctx, ggez::graphics::Color::RGBA(0, 0, 0, 0));
        Ok(state)
    }
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()> {
        let seconds = timer::duration_to_f64(dt);
        self.particles.update(seconds);
        println!("Particles: {}, FPS: {}", self.particles.count(), timer::get_fps(ctx));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let dest_rect = ggez::graphics::Rect::new(WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2, 0, 0);
        graphics::draw(ctx, &self.particles, None, Some(dest_rect))?;

        graphics::present(ctx);
        timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Shiny particles".to_string();
    c.window_width = WINDOW_WIDTH as u32;
    c.window_height = WINDOW_HEIGHT as u32;
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
