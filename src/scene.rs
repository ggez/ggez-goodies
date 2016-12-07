//! The idea here is basically we provide a Game type,
//! which can swap between several different Scene types.
//! Ideally Scenes can be nested and we can build a stack
//! of them?  Or something.
//! We also need hooks: Load, unload... more finely grained?
//! Kinda tricky to separate create/destroy vs. load and unload,
//! KISS for now.

use std::collections::BTreeMap;
use std::marker::Sized;

use ggez::GameResult;

struct SceneStateTest {
    state: i32,
}

trait SceneState {
    fn load_scene(self) -> Box<Scene<Self>>;
    fn unload_scene(s: Scene<Self>) -> Box<Self>;
    /*
    fn load_scene<S>(self) -> Box<S> where S: Scene + Sized  {
        S::load(Box::new(self))
    }
    
    fn unload_scene<S>(mut s: S) -> Self where S: Scene + Sized {
        s.unload()
    }
    */
}


trait Scene<S> where S: SceneState {
    //type S;
    fn update(&mut self) -> GameResult<()>;

    fn draw(&mut self) -> GameResult<()>;

    fn unload(&mut self) -> Box<S>;

    fn load(state: Box<S>) -> Box<Self> where Self: Sized;

    /*
    fn load_from_default() -> GameResult<Self>
        where Self: Sized,
              S: Default {
        let state = S::default();
        Self::load(state)
    }
*/
}
/*
pub trait SceneState<S> where S: Scene {
    fn load_scene(self) -> GameResult<S>;
    fn unload_scene(scene: S) -> GameResult<Self>
        where Self: Sized;
}

use std::any::Any;


// Okay, the basic semantics are:
// A scene gets loaded and passed some state, which can be a default
// So what we need to actually store are a name, a scene constructor,
// and the state for that scene.
// The state can be different for every scene as far as the SceneRunner
// is concerned, it's only associated with that one type of scene.
struct SceneRunner {
    scenes: BTreeMap<String, Box<SceneState<i32>>>,
}


impl SceneRunner {
    fn add_scene<S>(&mut self, name: &str, initial_state: SceneState<S>) {
        self.scenes.insert(name.to_string(), Box::new(initial_state));
    }
}
*/

/*
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
*/
