
use ggez;
use ggez::GameResult;
use ggez::conf;
use ggez::event;
use ggez::game::GameState;

use std::collections::BTreeMap;
use std::time::Duration;


pub trait SavedScene<T> {
    fn load(&self) -> Box<Scene<T>>;
    fn name(&self) -> &str;
}

pub trait Scene<T> {
    fn unload(&mut self) -> Box<SavedScene<T>>;

    /// Called upon each physics update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self,
              ctx: &mut ggez::Context,
              dt: Duration,
              scenes: &mut SceneStore<T>)
              -> GameResult<Option<String>>;

    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// `graphics::clear()` and end it with
    /// `graphics::present()` and `timer::sleep_until_next_frame()`
    fn draw(&mut self, ctx: &mut ggez::Context, scenes: &mut SceneStore<T>) -> GameResult<()>;

    // You don't have to override these if you don't want to; the defaults
    // do nothing.
    // It might be nice to be able to have custom event types and a map or
    // such of handlers?  Hmm, maybe later.
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

pub struct SceneStore<T> {
    states: BTreeMap<String, Box<SavedScene<T>>>,
    pub game_data: T,
}

impl<T> SceneStore<T> {
    pub fn add<S: SavedScene<T> + 'static>(&mut self, scene_state: S) {
        self.states.insert(scene_state.name().to_string(), Box::new(scene_state));
    }
}

/// A SceneManager is a GameState that handles Scene's
/// and switches from one to another when requested.
///
/// The stuff you would normally store in your GameState
/// type should implement GameData and go into the T type.
pub struct SceneManager<T> {
    store: SceneStore<T>,
    current: Box<Scene<T>>,
    next_scene: Option<String>,
}

impl<T> GameState for SceneManager<T>
    where T: GameData<T>
{
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> {
        let starting_scene_state = T::starting_scene();
        let game_data = T::load(ctx, conf)?;

        Ok(Self::new(starting_scene_state, game_data))
    }

    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        // TODO: Get rid of this hacky clone!
        if let Some(ref scene_name) = self.next_scene.clone() {
            self.switch_scene(&scene_name)?;
        }
        self.next_scene = self.current.update(ctx, dt, &mut self.store)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        self.current.draw(ctx, &mut self.store)
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
    fn new(starting_scene_state: Box<SavedScene<T>>, game_data: T) -> Self {
        let starting_scene = starting_scene_state.load();
        let mut scenes: BTreeMap<String, Box<SavedScene<T>>> = BTreeMap::new();
        scenes.insert(starting_scene_state.name().to_string(),
                      starting_scene_state);
        let store = SceneStore {
            states: scenes,

            game_data: game_data,
        };
        let sm = SceneManager {
            current: starting_scene,
            store: store,
            next_scene: None,
        };
        sm
    }

    pub fn current(&self) -> &Scene<T> {
        &*self.current
    }

    pub fn current_mut(&mut self) -> &mut Scene<T> {
        &mut *self.current
    }

    pub fn switch_scene(&mut self, scene_name: &str) -> GameResult<()> {
        // Save current scene
        let old_scene_state = self.current.unload();
        let old_scene_name = old_scene_state.name().to_string();
        self.store.states.insert(old_scene_name, old_scene_state);
        // Then load the new one.
        if let Some(scene_state) = self.store.states.get_mut(scene_name) {
            let new_scene = scene_state.load();
            self.current = new_scene;
            Ok(())
        } else {
            let msg = format!("SceneManager: Asked to load scene {} but it did not exist?",
                              scene_name);
            Err(ggez::GameError::ResourceNotFound(msg))
        }
    }
}

#[cfg(test)]
mod tests {

    use ggez;
    use ggez::GameResult;
    use ggez::game::GameState;

    use std::time::Duration;

    use super::{Scene, SavedScene, SceneManager, SceneStore};

    #[derive(Clone, Debug)]
    struct TestSavedScene {
        value: i32,
        name: String,
    }

    impl SavedScene<()> for TestSavedScene {
        fn load(&self) -> Box<Scene<()>> {
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


        fn update(&mut self,
                  _ctx: &mut ggez::Context,
                  _dt: Duration,
                  _scenes: &mut SceneStore<()>)
                  -> GameResult<Option<String>> {
            Ok(None)
        }

        fn draw(&mut self,
                _ctx: &mut ggez::Context,
                _scenes: &mut SceneStore<()>)
                -> GameResult<()> {
            Ok(())
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
        sm.store.add(new_scene);

        {
            let s = sm.current_mut().unload();
            assert_eq!(s.name(), "default scene");
        }

        let res = sm.switch_scene("other scene");
        assert!(res.is_ok());

        {
            let s = sm.current_mut().unload();
            assert_eq!(s.name(), "other scene");
        }

        let res = sm.switch_scene("non existent scene");
        assert!(res.is_err());
    }

}
