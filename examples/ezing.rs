extern crate ggez;
extern crate ezing;

use ggez::conf;
use ggez::event;
use ggez::{Context, GameResult};
use ggez::graphics;
use ggez::graphics::{Point2};
use ggez::timer;

use ezing::cubic_inout;

extern crate ggez_goodies;

struct Tween {
    t: f32,
    start: f32,
    end: f32,
}

fn interpolate(tween: &Tween) -> f32 {
    cubic_inout((tween.t - tween.start) / tween.end).min(1.0).max(0.0)
}

struct MainState {
    tween: Tween,
    image: graphics::Image
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let image = graphics::Image::solid(ctx, 50, graphics::Color::new(1.0, 0.0, 0.0, 1.0))?;
        let state = MainState { 
            image,
            tween: Tween {t: 0.0, start: 1.0, end: 3.0}
        };
        graphics::set_background_color(ctx, ggez::graphics::Color::from((0, 0, 0, 0)));
        Ok(state)
    }
}

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            let seconds = 1.0 / (DESIRED_FPS as f32);
            self.tween.t += seconds;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::draw(
            ctx,
            &mut self.image,
            Point2::new(WINDOW_WIDTH * interpolate(&self.tween) / 2.0, WINDOW_HEIGHT / 2.0),
            0.0,
        )?;
        graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_setup.title = "Smooth as Butter".to_string();
    c.window_mode.width = WINDOW_WIDTH as u32;
    c.window_mode.height = WINDOW_HEIGHT as u32;
    let ctx = &mut Context::load_from_conf("smooth_interpolates", "test", c).unwrap();
    let game = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, game) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
