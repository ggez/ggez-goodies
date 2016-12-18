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
use ggez::event;
use ggez::game::GameState;

use std::collections::BTreeMap;
use std::time::Duration;

/// Arbitrary data that can be loaded to produce a Scene
/// of a particular type.  The T is the global data that
/// the Scene relies upon to function, such as your global
/// game state.
pub trait SavedScene<T> {
    fn load(&mut self, _thingy: Thingy<T>) -> Box<Scene<T>>;
    fn name(&self) -> &str;
}

/// A Scene is sort of a stand-in GameState that can be
/// loaded and unloaded by the SceneManager.  When unloaded
/// it produces something implementing the SavedScene state,
/// which will re-produce the Scene when loaded.
///
/// Like a GameState, it has update() and draw() methods
/// (with a slightly different signature), as well as event
/// handlers.
pub trait Scene<T> {
    fn unload(&mut self) -> Box<SavedScene<T>>;

    /// Note this returns an Option<String>;
    /// if you want to switch to a new scene,
    /// return the name of the scene in the Option
    /// and it will be loaded.  Or None to just carry on.
    fn update(&mut self,
              _ctx: &mut ggez::Context,
              _dt: Duration)
              -> GameResult<Option<Box<Scene<T>>>> {
        Ok(None)
    }

    fn draw(&mut self, _ctx: &mut ggez::Context) -> GameResult<()> {
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _button: event::Mouse, _x: i32, _y: i32) {}

    fn mouse_button_up_event(&mut self, _button: event::Mouse, _x: i32, _y: i32) {}

    fn mouse_motion_event(&mut self,
                          _state: event::MouseState,
                          _x: i32,
                          _y: i32,
                          _xrel: i32,
                          _yrel: i32) {
    }

    fn mouse_wheel_event(&mut self, _x: i32, _y: i32) {}

    fn key_down_event(&mut self,
                      _keycode: Option<event::Keycode>,
                      _keymod: event::Mod,
                      _repeat: bool) {
    }

    fn key_up_event(&mut self,
                    _keycode: Option<event::Keycode>,
                    _keymod: event::Mod,
                    _repeat: bool) {
    }

    fn focus_event(&mut self, _gained: bool) {}

    /// Called upon a quit event.  If it returns true,
    /// the game does not exit.
    fn quit_event(&mut self) -> bool {
        println!("Quitting game");
        false
    }
}

/// The GameData trait just provides
/// a method to create a new object of type T, instantiating
/// your global game data.
///
/// It also provides a method that is called to generate
/// the first scene of your game.
pub trait GameData<T>
    where Self: Sized
{
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self>;
    fn starting_scene() -> Box<SavedScene<T>>;
}

/// A thingy that contains saved scenes and
/// arbitrary game data.
///
/// Basically this is being extracted from the
/// scene manager so that the data that scenes
/// actually need is explicitly made available
/// to them and passed from one to the other,
/// rather than being provided to them from
/// outside.
pub struct Thingy<T> {
    states: BTreeMap<String, Box<SavedScene<T>>>,
    game_data: T,
}

impl<T> Thingy<T> {
    fn new(data: T) -> Self {
        Thingy {
            states: BTreeMap::new(),
            game_data: data,
        }
    }


    pub fn add<S: SavedScene<T> + 'static>(&mut self, scene_state: S) {
        self.states.insert(scene_state.name().to_string(), Box::new(scene_state));
    }

    // pub fn switch_scene(&mut self, scene_name: &str) -> GameResult<()> {
    // Save current scene
    // let old_scene_state = self.current.unload();
    // let old_scene_name = old_scene_state.name().to_string();
    // self.states.insert(old_scene_name, old_scene_state);
    // if let Some(scene_state) = self.states.get_mut(scene_name) {
    // let new_scene = scene_state.load();
    // self.current = new_scene;
    // Ok(())
    // } else {
    // let msg = format!("SceneManager: Asked to load scene {} but it did not exist?",
    // scene_name);
    // Err(ggez::GameError::ResourceNotFound(msg))
    // }
    // }
    //
}

/// A SceneManager is a GameState that handles Scene's
/// and switches from one to another when requested.
///
/// The stuff you would normally store in your GameState
/// type should implement GameData and go into the T type.
pub struct SceneManager<T> {
    current: Box<Scene<T>>,
}

impl<T> GameState for SceneManager<T>
    where T: GameData<T>
{
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> {
        let starting_scene_state = T::starting_scene();
        Ok(Self::new(starting_scene_state))
    }

    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        // TODO: Get rid of this hacky clone!
        if let Some(newscene) = self.current.update(ctx, dt)? {
            self.current = newscene;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        self.current.draw(ctx)
    }

    fn mouse_button_down_event(&mut self, button: event::Mouse, x: i32, y: i32) {
        self.current.mouse_button_down_event(button, x, y)
    }

    fn mouse_button_up_event(&mut self, button: event::Mouse, x: i32, y: i32) {
        self.current.mouse_button_up_event(button, x, y)
    }

    fn mouse_motion_event(&mut self,
                          _state: event::MouseState,
                          _x: i32,
                          _y: i32,
                          _xrel: i32,
                          _yrel: i32) {
        self.current.mouse_motion_event(_state, _x, _y, _xrel, _yrel)
    }

    fn mouse_wheel_event(&mut self, _x: i32, _y: i32) {
        self.current.mouse_wheel_event(_x, _y)
    }

    fn key_down_event(&mut self,
                      _keycode: Option<event::Keycode>,
                      _keymod: event::Mod,
                      _repeat: bool) {
        self.current.key_down_event(_keycode, _keymod, _repeat)
    }

    fn key_up_event(&mut self,
                    _keycode: Option<event::Keycode>,
                    _keymod: event::Mod,
                    _repeat: bool) {
        self.current.key_up_event(_keycode, _keymod, _repeat)
    }

    fn focus_event(&mut self, _gained: bool) {
        self.current.focus_event(_gained)
    }

    /// Called upon a quit event.  If it returns true,
    /// the game does not exit.
    fn quit_event(&mut self) -> bool {
        self.current.quit_event()
    }
}

impl<T> SceneManager<T> {
    /// This lets us create a SceneManager by providing the data for it,
    /// instead of having it implicitly created via the GameData trait.
    fn new(mut starting_scene_state: Box<SavedScene<T>>, thingy: Thingy<T>) -> Self {
        let starting_scene = starting_scene_state.load(thingy);
        let sm = SceneManager { current: starting_scene };
        sm
    }
}

#[cfg(test)]
mod tests {
    use super::{Scene, SavedScene, SceneManager};

    #[derive(Clone, Debug)]
    struct TestSavedScene {
        value: i32,
        name: String,
    }

    impl SavedScene<()> for TestSavedScene {
        fn load(&mut self) -> Box<Scene<()>> {
            Box::new(TestScene(self.clone()))
        }
        fn name(&self) -> &str {
            &self.name
        }
    }

    #[derive(Clone, Debug)]
    struct TestScene(TestSavedScene);

    impl Scene<()> for TestScene {
        fn unload(&mut self) -> Box<SavedScene<()>> {
            Box::new(self.0.clone())
        }
    }

    #[test]
    fn test_scene_switching() {
        let default_scene = TestSavedScene {
            name: "default scene".to_string(),
            value: 42,
        };
        let new_scene = TestSavedScene {
            name: "other scene".to_string(),
            value: 23,
        };
        let mut sm = SceneManager::new(Box::new(default_scene), ());
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
