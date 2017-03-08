extern crate ggez;
extern crate ggez_goodies;
use ggez::conf;
use ggez::event;
use ggez::GameResult;
use ggez::graphics;
use ggez::timer;
use std::time::Duration;

/*
use ggez_goodies::scene::*;

struct MainState {
    font: graphics::Font,
    message_text: graphics::Text,
}

/// A bootstrap scene whose only purpose is to
/// load all the other bits necessary...
struct StartScene;
impl SavedScene<MainState> for StartScene {
    fn load(&self) -> Box<Scene<MainState>> {
        Box::new(StartScene)
    }
    fn name(&self) -> &str {
        "Starting Scene"
    }
}

impl Scene<MainState> for StartScene {
    fn unload(&mut self) -> Box<SavedScene<MainState>> {
        Box::new(StartScene)
    }


    fn update(&mut self,
              _ctx: &mut ggez::Context,
              _dt: Duration,
              state: &mut SceneStore<MainState>)
              -> GameResult<Option<String>> {
        let s1 = SavedScene1::new("Scene 1", "Scene 2");
        let s2 = SavedScene1::new("Scene 2", "Scene 1");
        state.add(s1);
        state.add(s2);
        Ok(Some("Scene 1".to_string()))
    }

    fn draw(&mut self,
            _ctx: &mut ggez::Context,
            _store: &mut SceneStore<MainState>)
            -> GameResult<()> {
        Ok(())
    }
}

impl GameData<MainState> for MainState {
    fn load(ctx: &mut ggez::Context, _conf: &conf::Conf) -> GameResult<Self> {
        let font = graphics::Font::new(ctx, "DejaVuSerif.ttf", 16)?;

        let text = graphics::Text::new(ctx, "Press space to switch to the next scene.", &font)?;
        Ok(MainState {
            font: font,
            message_text: text,
        })
    }
    fn starting_scene() -> Box<SavedScene<MainState>> {
        Box::new(StartScene)
    }
}

#[derive(Clone, Debug)]
struct SavedScene1 {
    time_unloaded: f64,
    name: String,
    next_scene: String,
}

impl SavedScene1 {
    fn new(name: &str, next_scene: &str) -> Self {
        SavedScene1 {
            time_unloaded: 0.0,
            name: name.to_string(),
            next_scene: next_scene.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct Scene1 {
    current_time: f64,
    name: String,
    next_scene: String,
    switch_to_next: bool,
}


impl SavedScene<MainState> for SavedScene1 {
    fn load(&self) -> Box<Scene<MainState>> {
        Box::new(Scene1 {
            current_time: self.time_unloaded,
            name: self.name.clone(),
            next_scene: self.next_scene.clone(),
            switch_to_next: false,
        })
    }
    fn name(&self) -> &str {
        &self.name
    }
}

impl Scene<MainState> for Scene1 {
    fn unload(&mut self) -> Box<SavedScene<MainState>> {
        Box::new(SavedScene1 {
            time_unloaded: self.current_time,
            name: self.name.clone(),
            next_scene: self.next_scene.clone(),
        })
    }


    fn update(&mut self,
              _ctx: &mut ggez::Context,
              dt: Duration,
              _state: &mut SceneStore<MainState>)
              -> GameResult<Option<String>> {
        let seconds = timer::duration_to_f64(dt);
        self.current_time += seconds;
        if self.switch_to_next {
            Ok(Some(self.next_scene.clone()))
        } else {
            Ok(None)
        }
    }

    fn draw(&mut self,
            ctx: &mut ggez::Context,
            store: &mut SceneStore<MainState>)
            -> GameResult<()> {
        ctx.renderer.clear();
        let message = format!("Scene '{}' has been running for {:0.2} seconds",
                              self.name,
                              self.current_time);
        let state = &mut store.game_data;
        let text = &mut graphics::Text::new(ctx, &message, &state.font)?;
        let text_rect = graphics::Rect::new(10, 240, text.width(), text.height());

        try!(graphics::draw(ctx, text, None, Some(text_rect)));


        let text_rect2 = graphics::Rect::new(10,
                                             270,
                                             state.message_text.width(),
                                             state.message_text.height());

        try!(graphics::draw(ctx, &mut state.message_text, None, Some(text_rect2)));

        ctx.renderer.present();
        timer::sleep_until_next_frame(ctx, 60);
        Ok(())
    }

    fn key_down_event(&mut self,
                      keycode: Option<event::Keycode>,
                      _keymod: event::Mod,
                      _repeat: bool) {
        if let Some(event::Keycode::Space) = keycode {
            self.switch_to_next = true;

        }
    }
}
*/
pub fn main() {
    let c = conf::Conf::new();
    /*let mut game: Game<SceneManager<MainState>> = Game::new("scenetest", c).unwrap();
    if let Err(e) = game.run() {
        println!("Error encountered: {:?}", e);
    } else {
        println!("Game exited cleanly.");
    }
*/
}
