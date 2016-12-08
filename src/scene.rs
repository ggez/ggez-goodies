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
use std::collections::btree_map::Entry;
use std::time::Duration;

trait SceneState {
    fn load(&mut self) -> Box<Scene>;
    fn name(&self) -> String;
}

trait Scene {
    fn unload(&mut self) -> Box<SceneState>;
}

struct SceneManager {
    states: BTreeMap<String, Box<SceneState>>,
    current: Box<Scene>,
}

impl SceneManager {
    pub fn switch_scene(&mut self, scene_name: &str) -> GameResult<()> {
        // Save current scene
        let old_scene_state = self.current.unload();
        self.states.insert(old_scene_state.name(), old_scene_state);
        if let Some(scene_state) = self.states.get_mut(scene_name) {
            let new_scene = scene_state.load();
            self.current = new_scene;
            Ok(())
        } else {
            let msg = format!("SceneManager: Asked to load scene {} but it did not exist?", scene_name);
            Err(ggez::GameError::ResourceNotFound(msg))
        }
    }

    fn new<S: SceneState + 'static>(mut default_scene: S) -> Self {
        let new_scene = default_scene.load();
        let mut scenes: BTreeMap<String, Box<SceneState>> = BTreeMap::new();
        scenes.insert(default_scene.name(), Box::new(default_scene));
        SceneManager {
            current: new_scene,
            states: scenes,
        }
    }

    fn add<S: SceneState + 'static>(&mut self, scene_state: S) {
        self.states.insert(scene_state.name(), Box::new(scene_state));
    }

    fn current(&self) -> &Scene {
        &*self.current
    }

    fn current_mut(&mut self) -> &mut Scene {
        &mut *self.current
    }

}

/*
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
*/

mod tests {
    use super::{Scene, SceneState, SceneManager};
    
    #[derive(Clone, Debug)]
    struct TestSceneState {
        value: i32,
        name: String,
    }

    impl SceneState for TestSceneState {
        fn load(&mut self) -> Box<Scene> {
            Box::new(TestScene(self.clone()))
        }
        fn name(&self) -> String {
            self.name.clone()
        }
    }

    #[derive(Clone, Debug)]
    struct TestScene(TestSceneState);

    impl Scene for TestScene {
        fn unload(&mut self) -> Box<SceneState> {
            Box::new(self.0.clone())
        }
    }

    #[test]
    fn test_scene_switching() {
        let default_scene = TestSceneState{name: "default scene".to_string(), value: 42};
        let new_scene = TestSceneState{name: "other scene".to_string(), value: 23};
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
