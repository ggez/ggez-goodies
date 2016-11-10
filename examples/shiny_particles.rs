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

struct MainState {

}

impl GameState for MainState {
    fn load(ctx: &mut Context, conf: &conf::Conf) -> GameResult<Self> {
        let state = MainState {};
        Ok(state)
    }
    fn update(&mut self, ctx: &mut Context, dt: Duration) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        
        graphics::present(ctx);
        timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Shiny particles".to_string();
    c.window_width = 640;
    c.window_height = 480;
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
