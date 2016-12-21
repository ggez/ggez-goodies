extern crate ggez;
extern crate ggez_goodies;
use ggez::conf;
use ggez::event;
use ggez::game::{Game, GameState};
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::timer;
use std::time::Duration;

use ggez_goodies::scene::*;

struct MainState {
    font: graphics::Font,
    message_text: graphics::Text,
}

impl GameData for MainState {
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> {
        let font = graphics::Font::new(ctx, "DejaVuSerif.ttf", 16)?;

        let text = graphics::Text::new(ctx, "Press space to switch to the next scene.", &font)?;
        let scene = SavedScene1 {
            time_unloaded: 0.0,
            name: "Starting scene".to_string(),
        };
        Ok(MainState {
            font: font,
            message_text: text,
        })
    }
    fn starting_scene() -> Box<SavedScene> {

        let scene = SavedScene1 {
            time_unloaded: 0.0,
            name: "Starting scene".to_string(),
        };
        Box::new(scene)
    }
}

#[derive(Clone, Debug)]
struct SavedScene1 {
    time_unloaded: f64,
    name: String,
}

#[derive(Clone, Debug)]
struct Scene1 {
    current_time: f64,
    name: String,
}


impl SavedScene for SavedScene1 {
    fn load(&self) -> Box<Scene> {
        Box::new(Scene1 {
            current_time: self.time_unloaded,
            name: self.name.clone(),
        })
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl Scene for Scene1 {
    fn unload(&mut self) -> Box<SavedScene> {
        Box::new(SavedScene1 {
            time_unloaded: self.current_time,
            name: self.name.clone(),
        })
    }


    fn update(&mut self,
              _ctx: &mut ggez::Context,
              dt: Duration,
              _state: &mut SceneStore)
              -> GameResult<Option<String>> {
        let seconds = timer::duration_to_f64(dt);
        self.current_time += seconds;
        Ok(None)
    }

    fn draw(&mut self, ctx: &mut ggez::Context, state: &mut SceneStore) -> GameResult<()> {
        ctx.renderer.clear();
        let message = format!("Scene '{}' has been running for {:0.2} seconds",
                              self.name,
                              self.current_time);
        // let text = &mut graphics::Text::new(ctx, &message, &state.font)?;
        // let text_rect = graphics::Rect::new(10, 240, text.width(), text.height());
        //
        // try!(graphics::draw(ctx, text, None, Some(text_rect)));
        //
        //
        // let text_rect2 = graphics::Rect::new(10,
        // 270,
        // state.message_text.width(),
        // state.message_text.height());
        //
        // try!(graphics::draw(ctx, &mut state.message_text, None, Some(text_rect2)));
        //
        ctx.renderer.present();
        timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }

    fn key_down_event(&mut self,
                      _keycode: Option<event::Keycode>,
                      _keymod: event::Mod,
                      _repeat: bool) {
        println!("Key pressed!");

    }
}
// impl GameState for MainState {
// fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self>
// where Self: Sized
// {
// let font = graphics::Font::new(ctx, "DejaVuSerif.ttf", 16)?;
//
// let text = graphics::Text::new(ctx, "Press space to switch to the next scene.", &font)?;
// let scene = SavedScene1 {
// time_unloaded: 0.0,
// name: "Starting scene".to_string(),
// };
// let sm = SceneManager::new(Box::new(scene));
// Ok(MainState {
// font: font,
// message_text: text,
// scenes: sm,
// })
// }
//
//
// fn update(&mut self, _ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
// self.scenes.update(_ctx, dt, self)
// }
//
// fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
// self.scenes.draw(ctx, self)
// }
// }
//
pub fn main() {
    let c = conf::Conf::new();
    let mut game: Game<SceneManager<MainState>> = Game::new("scenetest", c).unwrap();
    if let Err(e) = game.run() {
        println!("Error encountered: {:?}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
