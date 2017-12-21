//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;


use ggez::conf;
use ggez::event;
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::graphics::{Point2, Vector2};
use ggez::timer;
use std::time::Duration;
use ggez::nalgebra as na;

extern crate ggez_goodies;
use ggez_goodies::particle::*;

struct MainState {
    particles: ParticleSystem,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let system = ParticleSystemBuilder::new(ctx)
            .count(5000)
            .acceleration(Vector2::new(0.0, 50.0))
            .start_max_age(5.0)
            .start_size_range(2.0, 15.0)
            .start_color_range(graphics::Color::from((0, 0, 0)),
                               graphics::Color::from((255, 255, 255)))
            .start_velocity_range(Vector2::new(-50.0, -200.0), Vector2::new(50.0, 0.0))
            .start_rotation_range(-10.0, 10.0)
            .emission_rate(100.0)
            .delta_size(Transition::range(15.0, 5.0))
            .delta_color(Transition::range(ggez::graphics::Color::from((255, 0, 0)),
                                           ggez::graphics::Color::from((255, 255, 0))))
            //.emission_shape(EmissionShape::Circle(Point2::new(0.0, 0.0), 150.0))
            .emission_shape(EmissionShape::Line(Point2::new(-100.0, -100.0), Point2::new(100.0, 100.0)))
            .build();
        let state = MainState { particles: system };
        graphics::set_background_color(ctx, ggez::graphics::Color::from((0, 0, 0, 0)));
        Ok(state)
    }
}

// const WINDOW_WIDTH: i32 = 640;
// const WINDOW_HEIGHT: i32 = 480;

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // let seconds = timer::duration_to_f64(dt);
        let seconds = 1.0 / 60.0;

        self.particles.update(seconds);
        println!("Particles: {}, FPS: {}",
                 self.particles.count(),
                 timer::get_fps(ctx));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        //let dest_rect = ggez::graphics::Rect::new(WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2, 0, 0);
        //graphics::draw(ctx, &mut self.particles, None, Some(dest_rect))?;

        graphics::present(ctx);
        // timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_title = "Shiny particles".to_string();
    // c.window_width = WINDOW_WIDTH as u32;
    // c.window_height = WINDOW_HEIGHT as u32;
    let ctx = &mut Context::load_from_conf("shiny_particles", "test", c).unwrap();
    let game = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, game) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
