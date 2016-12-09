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
    data: i32,
}

struct SceneState1 {
    data: i32,
}

struct Scene1 {
    data: i32,
}


impl SceneState<MainState> for SceneState1 {
    fn load(&mut self) -> Box<Scene<MainState>> {
        Box::new(Scene1 { data: self.data })
    }
    fn name(&self) -> String {
        "Test state".to_string()
    }
}

impl Scene<MainState> for Scene1 {
    fn unload(&mut self) -> Box<SceneState<MainState>> {
        Box::new(SceneState1 { data: self.data })
    }
}


impl Loadable<MainState> for MainState {
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self>
        where Self: Sized
    {
        Ok(MainState { data: 0 })
    }
    fn default_scene() -> Box<SceneState<MainState> + 'static> {
        Box::new(SceneState1 { data: 1 })
    }
}
// impl GameState for MainState {
// fn load(ctx: &mut Context, _conf: &conf::Conf) -> GameResult<MainState> {
// let font = graphics::Font::new(ctx, "DejaVuSerif.ttf", 48).unwrap();
// let text = graphics::Text::new(ctx, "Hello world!", &font).unwrap();
//
// let s = MainState { data: 1 };
// Ok(s)
// }
//
// fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
// Ok(())
// }
//
// fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
// ctx.renderer.clear();
// try!(graphics::draw(ctx, &mut self.text, None, None));
// ctx.renderer.present();
// timer::sleep_until_next_frame(ctx, 60);
// Ok(())
// }
// }
//

pub fn main() {
    let c = conf::Conf::new();
    let mut game: Game<SceneManager<MainState>> = Game::new("helloworld", c).unwrap();
    if let Err(e) = game.run() {
        println!("Error encountered: {:?}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
