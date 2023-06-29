use ggez::{
    self, conf, event,
    graphics::{self, Color},
    timer, Context, GameResult,
};
use ggez_goodies::{self, euclid as eu, tilemap as t};

struct MainState {
    tilemap: t::Map,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut image = graphics::Image::from_path(ctx, "/terrain.png")?;
        let tiled_map = {
            use std::io::Read;
            let mut f = ctx.fs.open("/test-map.tmx")?;
            let buf = &mut vec![];
            f.read_to_end(buf)?;
            t::tiled::parse(buf.as_slice()).unwrap()
        };
        let tilemap = t::Map::from_tiled(ctx, tiled_map, &mut move |_ctx, _path| image.clone());
        let state = MainState { tilemap };
        Ok(state)
    }
}

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            timer::sleep(std::time::Duration::from_secs(0));
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        let dest: ggez_goodies::Point2 = eu::point2(WINDOW_WIDTH / 4.0, WINDOW_HEIGHT / 4.0);
        canvas.draw(&self.tilemap, graphics::DrawParam::default().dest(dest));
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

    let (mut ctx, event_loop) = ggez::ContextBuilder::new("tile_map", "test")
        .window_setup(conf::WindowSetup::default().title("Tile it like it's 1988 again!"))
        .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .add_resource_path(resource_dir)
        .build()?;

    let game = MainState::new(&mut ctx)?;

    event::run(ctx, event_loop, game)
}
