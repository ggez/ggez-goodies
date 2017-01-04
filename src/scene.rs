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

pub trait SavedScene<T> {
    fn load(&mut self) -> Box<Scene<T>>;
    fn name(&self) -> &str;
}

pub trait Scene<T> {
    fn unload(&mut self) -> Box<SavedScene<T>>;

    fn update(&mut self,
              _ctx: &mut ggez::Context,
              _dt: Duration,
              _state: &mut T)
              -> GameResult<Option<String>> {
        Ok(None)
    }

    fn draw(&mut self, _ctx: &mut ggez::Context, _state: &mut T) -> GameResult<()> {
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

pub trait Loadable<T> {
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> where Self: Sized;
    fn default_scene() -> Box<SavedScene<T> + 'static>;
}

pub struct SceneManager<T> {
    states: BTreeMap<String, Box<SavedScene<T>>>,
    current: Box<Scene<T>>,
    game_data: T,
    next_scene: Option<String>,
}

impl<T> GameState for SceneManager<T>
    where T: Loadable<T>
{    
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> {
        let mut default_scene_state = T::default_scene();
        let game_data = T::load(ctx, conf)?;

        Ok(Self::new(default_scene_state, game_data))
    }

    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()> {
        // TODO: Get rid of this hacky clone!
        if let Some(scene_name) = self.next_scene.clone() {
            self.switch_scene(&scene_name);
        }
        self.next_scene = self.current.update(ctx, dt, &mut self.game_data)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        self.current.draw(ctx, &mut self.game_data)
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

impl<T> SceneManager<T>
{
    /// This lets us create a SceneManager by providing the data for it,
    /// instead of having it implicitly created via the Loadable trait.
    fn new(mut default_scene_state: Box<SavedScene<T>>, game_data: T) -> Self {
        let default_scene = default_scene_state.load();
        let mut scenes: BTreeMap<String, Box<SavedScene<T>>> = BTreeMap::new();
        scenes.insert(default_scene_state.name().to_string(), default_scene_state);
        let sm = SceneManager {
            current: default_scene,
            states: scenes,
            game_data: game_data,
            next_scene: None,
        };
        sm
    }

    pub fn add<S: SavedScene<T> + 'static>(&mut self, scene_state: S) {
        self.states.insert(scene_state.name().to_string(), Box::new(scene_state));
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
}

impl<T> SceneManager<T>
    where T: Loadable<T>
{
    
    // pub fn new<S: SavedScene + 'static>(mut default_scene: S) -> Self {
    // let new_scene = default_scene.load();
    // let mut scenes: BTreeMap<String, Box<SavedScene>> = BTreeMap::new();
    // scenes.insert(default_scene.name(), Box::new(default_scene));
    // SceneManager {
    // current: new_scene,
    // states: scenes,
    // }
    // }
    //
/*
    pub fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<Option<String>> {
        self.current.update(ctx, dt, &mut self.game_data)?;
        Ok(None)        
    }

    pub fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()> {
        self.current.draw(ctx, &mut self.game_data)
    }
*/
}

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
        
        let res = sm.switch_scene("other scene").unwrap();
        {
            let mut s = sm.current_mut().unload();
            assert_eq!(s.name(), "other scene");
        }
        
        let res = sm.switch_scene("non existent scene");
        assert!(res.is_err());
    }
    
}
