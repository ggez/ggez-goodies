extern crate ggez;
extern crate ggez_goodies;
use ggez::conf;
use ggez::game::{Game, GameState};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::timer;
use std::time::Duration;

use ggez_goodies::scene::*;

// First we make a structure to contain the game's state
struct MainState {
    font: graphics::Font,
}

struct SceneState1 {
    time_unloaded: f64,
}

struct Scene1 {
    current_time: f64,
}


impl SceneState<MainState> for SceneState1 {
    fn load(&mut self) -> Box<Scene<MainState>> {
        Box::new(Scene1 { current_time: 0.0 })
    }
    fn name(&self) -> String {
        "Test state".to_string()
    }
}

impl Scene<MainState> for Scene1 {
    fn unload(&mut self) -> Box<SceneState<MainState>> {
        Box::new(SceneState1 { time_unloaded: 0.0 })
    }


    fn update(&mut self,
              _ctx: &mut ggez::Context,
              _dt: Duration,
              _state: &mut MainState)
              -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context, state: &mut MainState) -> GameResult<()> {
        ctx.renderer.clear();
        let text = &mut graphics::Text::new(ctx, "Hello world!", &state.font)?;
        try!(graphics::draw(ctx, text, None, None));
        ctx.renderer.present();
        timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }
}


impl Loadable<MainState> for MainState {
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self>
        where Self: Sized
    {
        let font = graphics::Font::new(ctx, "DejaVuSerif.ttf", 48)?;
        Ok(MainState { font: font })
    }
    fn default_scene() -> Box<SceneState<MainState> + 'static> {
        Box::new(SceneState1 { time_unloaded: 0.0 })
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let mut game: Game<SceneManager<MainState>> = Game::new("scenetest", c).unwrap();
    if let Err(e) = game.run() {
        println!("Error encountered: {:?}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
