extern crate ggez;
extern crate rand;


use ggez::conf;
use ggez::event;
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::graphics::{Point2, Vector2};
use ggez::timer;

extern crate ggez_goodies;
use ggez_goodies::particle::*;

struct MainState {
    particles: ParticleSystem,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let system = ParticleSystemBuilder::new(ctx)
            .count(100000)
            .emission_rate(20000.0)
            .acceleration(Vector2::new(0.0, 50.0))
            .start_max_age(5.0)
            .start_size_range(2.0, 15.0)
            .start_color_range(graphics::Color::from((0, 0, 0)),
                               graphics::Color::from((255, 255, 255)))
            .start_velocity_range(Vector2::new(-50.0, -200.0), Vector2::new(50.0, 0.0))
            .start_ang_vel_range(-10.0, 10.0)
            .delta_size(Transition::range(15.0, 5.0))
            .delta_color(Transition::range(ggez::graphics::Color::from((255, 0, 0)),
                                           ggez::graphics::Color::from((255, 255, 0))))
            .emission_shape(EmissionShape::Circle(Point2::new(0.0, 0.0), 150.0))
            //.emission_shape(EmissionShape::Line(Point2::new(-100.0, -100.0), Point2::new(100.0, 100.0)))
            .build();
        let state = MainState { particles: system };
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
            self.particles.update(seconds);
            if timer::get_ticks(ctx) % 10 == 0 {
                println!("Particles: {}, FPS: {}",
                         self.particles.count(),
                         timer::get_fps(ctx));
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::draw(ctx, &mut self.particles, Point2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0), 0.0)?;
        //graphics::present(ctx);
        Ok(())
    }
}

pub fn main() {
    let mut c = conf::Conf::new();
    c.window_setup.title = "Shiny particles".to_string();
    c.window_mode.width = WINDOW_WIDTH as u32;
    c.window_mode.height = WINDOW_HEIGHT as u32;
    let ctx = &mut Context::load_from_conf("shiny_particles", "test", c).unwrap();
    let game = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, game) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
