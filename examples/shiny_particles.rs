use ggez::{
    self, conf, event,
    graphics::{self, Color},
    Context, GameResult,
};
use ggez_goodies::{self, euclid as eu, particle2 as p};

struct MainState {
    particles: p::ParticleSystem<p::DefaultParticle>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        /*
        let system = ParticleSystemBuilder::new(ctx)
            .count(10000)
            .emission_rate(2000.0)
            .acceleration(Vector2 { x: 0.0, y: 50.0 })
            .start_max_age(5.0)
            .start_size_range(2.0, 15.0)
            .start_color_range(
                graphics::Color::from((0, 0, 0)),
                graphics::Color::from((255, 255, 255)),
            )
            .start_velocity_range(
                Vector2 {
                    x: -50.0,
                    y: -200.0,
                },
                Vector2 { x: 50.0, y: 0.0 },
            )
            .start_ang_vel_range(-10.0, 10.0)
            .delta_size(Transition::range(15.0, 5.0))
            .delta_color(Transition::range(
                ggez::graphics::Color::from((255, 0, 0)),
                ggez::graphics::Color::from((255, 255, 0)),
            ))
            .emission_shape(EmissionShape::Circle(Point2 { x: 0.0, y: 0.0 }, 150.0))
            //.emission_shape(EmissionShape::Line(Point2::new(-100.0, -100.0), Point2::new(100.0, 100.0)))
            .build();
         */
        let image = graphics::Image::from_path(ctx, "/player.png")?;
        let emitter = p::Emitter::new(10.0);
        let particles = p::ParticleSystem::new(1000, emitter, image, ctx);
        let state = MainState { particles };
        Ok(state)
    }
}

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            let seconds = 1.0 / (DESIRED_FPS as f32);
            self.particles.update(seconds);
            if ctx.time.ticks() % 10 == 0 {
                println!(
                    "Particles: {}, FPS: {}",
                    self.particles.count(),
                    ctx.time.fps()
                );
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        let dest: ggez_goodies::Point2 = eu::point2(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0);
        canvas.draw(&self.particles, graphics::DrawParam::new().dest(dest));
        canvas.finish(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    use std::env;
    use std::path;
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (mut ctx, event_loop) = ggez::ContextBuilder::new("shiny_particles", "test")
        .window_setup(conf::WindowSetup::default().title("Shiny particles"))
        .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .add_resource_path(resource_dir)
        .build()?;

    let game = MainState::new(&mut ctx).unwrap();

    event::run(ctx, event_loop, game)
}
