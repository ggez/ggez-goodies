//! The idea here is basically we provide a Game type,
//! which can swap between several different Scene types.
//! Ideally Scenes can be nested and we can build a stack
//! of them?  Or something.
//! We also need hooks: Load, unload... more finely grained?
//! Kinda tricky to separate create/destroy vs. load and unload,
//! KISS for now.

use ggez;
use ggez::GameResult;
use ggez::conf;
use ggez::game::GameState;

use std::time::Duration;

pub trait Scene: GameState {}

pub struct SceneManager {
    current_scene: Option<Box<Scene>>,
    scene_stack: Vec<Box<Scene>>,
}

impl SceneManager {
    pub fn switch_scene<T>(&mut self, ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<()>
        where T: Scene + 'static
    {
        let new_scene = T::load(ctx, conf)?;
        // let old_scene = &mut self.current_scene;
        self.current_scene = Some(Box::new(new_scene));
        Ok(())

    }
}


impl Default for SceneManager {
    fn default() -> Self {
        SceneManager {
            current_scene: None,
            scene_stack: Vec::default(),
        }
    }
}

impl GameState for SceneManager {
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> {
        Ok(SceneManager::default())
    }


    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        Ok(())
    }
}
