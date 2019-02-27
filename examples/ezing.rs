extern crate ezing;
extern crate ggez;

use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra::Point2;
use ggez::timer;
use ggez::{Context, GameResult};

use ezing::cubic_inout;

extern crate ggez_goodies;

struct Tween {
    t: f32,
    start: f32,
    end: f32,
}

fn interpolate(tween: &Tween) -> f32 {
    cubic_inout((tween.t - tween.start) / tween.end)
        .min(1.0)
        .max(0.0)
}

struct MainState {
    tween: Tween,
    image: graphics::Image,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let image = graphics::Image::solid(ctx, 50, graphics::Color::new(1.0, 0.0, 0.0, 1.0))?;
        let state = MainState {
            image,
            tween: Tween {
                t: 0.0,
                start: 1.0,
                end: 3.0,
            },
        };
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
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        graphics::draw(
            ctx,
            &mut self.image,
            (Point2::new(
                WINDOW_WIDTH * interpolate(&self.tween) / 2.0,
                WINDOW_HEIGHT / 2.0,
            ),),
        )?;
        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("smooth_interpolates", "test")
        .window_setup(conf::WindowSetup::default().title("Smooth as Butter"))
        .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()
        .unwrap();

    let game = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, event_loop, game) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
