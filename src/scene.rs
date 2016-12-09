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

pub trait SceneState<T> {
    fn load(&mut self) -> Box<Scene<T>>;
    fn name(&self) -> &str;
}

pub trait Scene<T> {
    fn unload(&mut self) -> Box<SceneState<T>>;

    fn update(&mut self,
              _ctx: &mut ggez::Context,
              _dt: Duration,
              _state: &mut T)
              -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut ggez::Context, _state: &mut T) -> GameResult<()> {
        Ok(())
    }
}

pub trait Loadable<T> {
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> where Self: Sized;
    fn default_scene() -> Box<SceneState<T> + 'static>;
}

pub struct SceneManager<T: Loadable<T>> {
    states: BTreeMap<String, Box<SceneState<T>>>,
    current: Box<Scene<T>>,
    game_data: T,
}

impl<T> GameState for SceneManager<T>
    where T: Loadable<T>
{
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> {
        let mut default_scene_state = T::default_scene();
        let default_scene = default_scene_state.load();
        let mut scenes: BTreeMap<String, Box<SceneState<T>>> = BTreeMap::new();
        scenes.insert(default_scene_state.name().to_string(), default_scene_state);
        let game_data = T::load(ctx, conf)?;
        let sm = SceneManager {
            current: default_scene,
            states: scenes,
            game_data: game_data,
        };
        Ok(sm)
    }

    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        self.current.update(ctx, dt, &mut self.game_data)
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        self.current.draw(ctx, &mut self.game_data)
    }
}

impl<T> SceneManager<T>
    where T: Loadable<T>
{
    pub fn switch_scene(&mut self, scene_name: &str) -> GameResult<()> {
        // Save current scene
        let old_scene_state = self.current.unload();
        let old_scene_name = old_scene_state.name().to_string();
        self.states.insert(old_scene_name, old_scene_state);
        if let Some(scene_state) = self.states.get_mut(scene_name) {
            let new_scene = scene_state.load();
            self.current = new_scene;
            Ok(())
        } else {
            let msg = format!("SceneManager: Asked to load scene {} but it did not exist?",
                              scene_name);
            Err(ggez::GameError::ResourceNotFound(msg))
        }
    }
    // pub fn new<S: SceneState + 'static>(mut default_scene: S) -> Self {
    // let new_scene = default_scene.load();
    // let mut scenes: BTreeMap<String, Box<SceneState>> = BTreeMap::new();
    // scenes.insert(default_scene.name(), Box::new(default_scene));
    // SceneManager {
    // current: new_scene,
    // states: scenes,
    // }
    // }
    //

    pub fn add<S: SceneState<T> + 'static>(&mut self, scene_state: S) {
        self.states.insert(scene_state.name().to_string(), Box::new(scene_state));
    }

    pub fn current(&self) -> &Scene<T> {
        &*self.current
    }

    pub fn current_mut(&mut self) -> &mut Scene<T> {
        &mut *self.current
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        self.current.update(ctx, dt, &mut self.game_data)
    }

    pub fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        self.current.draw(ctx, &mut self.game_data)
    }
}

// impl GameState for SceneManager {
// fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> {
// Ok(SceneManager::default())
// }
//
//
// fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
// Ok(())
// }
//
// fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
// Ok(())
// }
// }
//

mod tests {
    use super::{Scene, SceneState, SceneManager};

    #[derive(Clone, Debug)]
    struct TestSceneState {
        value: i32,
        name: String,
    }

    impl SceneState<()> for TestSceneState {
        fn load(&mut self) -> Box<Scene<()>> {
            Box::new(TestScene(self.clone()))
        }
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[derive(Clone, Debug)]
    struct TestScene(TestSceneState);

    impl Scene<()> for TestScene {
        fn unload(&mut self) -> Box<SceneState<()>> {
            Box::new(self.0.clone())
        }
    }

    #[test]
    fn test_scene_switching() {
        let default_scene = TestSceneState {
            name: "default scene".to_string(),
            value: 42,
        };
        let new_scene = TestSceneState {
            name: "other scene".to_string(),
            value: 23,
        };
        let mut sm = SceneManager::new(default_scene);
        sm.add(new_scene);

        {
            let mut s = sm.current_mut().unload();
            assert_eq!(s.name(), "default scene");
        }

        let res = sm.switch_scene("other scene");
        assert!(res.is_ok());

        {
            let mut s = sm.current_mut().unload();
            assert_eq!(s.name(), "other scene");
        }

        let res = sm.switch_scene("non existent scene");
        assert!(res.is_err());
    }

}
