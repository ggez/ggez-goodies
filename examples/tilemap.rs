use ggez::{self, conf, event, graphics, timer, Context, GameResult};
use ggez::graphics::Drawable;
use ggez_goodies::{self, euclid as eu, tilemap as t};

struct MainState {
    tilemap: t::Map,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut image = graphics::Image::new(ctx, "/terrain.png")?;
        image.set_filter(graphics::FilterMode::Nearest);
        let tiled_map = {
            use std::io::Read;
            let mut f = ggez::filesystem::open(ctx, "/test-map.tmx")?;
            let buf = &mut vec![];
            f.read_to_end(buf)?;
            t::tiled::parse(buf.as_slice()).unwrap()
        };
        let tilemap = t::Map::from_tiled(ctx, tiled_map, &|_| image.clone());
        let state = MainState { tilemap };
        Ok(state)
    }
}

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            timer::sleep(std::time::Duration::from_secs(0));
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        let dest: ggez_goodies::Point2 = eu::point2(WINDOW_WIDTH / 4.0, WINDOW_HEIGHT / 4.0);
        graphics::draw(
            ctx,
            &mut self.tilemap,
            graphics::DrawParam::default().dest(dest),
        )?;
        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() {
    use std::env;
    use std::path;
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("tile_map", "test")
        .window_setup(conf::WindowSetup::default().title("Tile it like it's 1988 again!"))
        .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .add_resource_path(resource_dir)
        .build()
        .unwrap();

    let game = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, event_loop, game) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
