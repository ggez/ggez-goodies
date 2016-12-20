
use ggez;
use ggez::GameResult;
use ggez::conf;
use ggez::event;
use ggez::game::GameState;

use std::collections::BTreeMap;
use std::time::Duration;


pub struct SceneManager {
    
}

pub trait Scene {
    fn load(ctx: &mut ggez::Context, conf: &conf::Conf) -> GameResult<Self> where Self: Sized;

    /// Called upon each physics update to the game.
    /// This should be where the game's logic takes place.
    fn update(&mut self, ctx: &mut ggez::Context, dt: Duration) -> GameResult<()>;

    /// Called to do the drawing of your game.
    /// You probably want to start this with
    /// `graphics::clear()` and end it with
    /// `graphics::present()` and `timer::sleep_until_next_frame()`
    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult<()>;

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
