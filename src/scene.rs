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

use std::collections::BTreeMap;
use std::time::Duration;

trait SceneState {
    fn load(self) -> Box<Scene>;
}

trait Scene {
    fn save(self) -> Box<SceneState>;
}

struct SceneManager {
    scene_states: BTreeMap<String, Box<SceneState>>,
    current_scene: Option<Box<Scene>>,
}



impl SceneManager {
    pub fn switch_scene(&mut self, scene_name: &str) -> GameResult<()> {
        Ok(())
    }
}


impl Default for SceneManager {
    fn default() -> Self {
        SceneManager {
            current_scene: None,
            scene_states: BTreeMap::new(),
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
