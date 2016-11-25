//! An abstract input state object that gets fed user
//! events and updates itself based on a set of key
//! bindings.
//! The idea is threefold:
//!
//! * Have a layer of abstract key bindings rather than
//! looking at concrete event types
//! * Use this to be able to abstract away differences
//! between keyboards, joysticks and game controllers
//! (rather based on Unity3D),
//! * Do some tweening of input axes and stuff just for
//! fun maybe.
//!
//! Right now ggez doesn't handle joysticks or controllers
//! anyway, so.

use std::cmp::PartialOrd;
use std::hash::Hash;
use std::collections::BTreeMap;
use std::collections::HashMap;
use ggez::event::*;


// Okay, but how does it actually work?
// Basically we have to bind input events to buttons and axes.
// Input events can be keys, mouse buttons/motion, or eventually
// joystick/controller inputs.  Mouse delta can be mapped to axes too.
//
// https://docs.unity3d.com/Manual/ConventionalGameInput.html has useful
// descriptions of the exact behavior of axes.
//
// So to think about this more clearly, here are the default bindings:
//
// W, ↑: +Y axis
// A, ←: -X axis
// S, ↓: -Y axis
// D, →: +X axis
// Enter, z, LMB: Button 1
// Shift, x, MMB: Button 2
// Ctrl,  c, RMB: Button 3
//
// Easy way?  Hash map of event -> axis/button bindings.

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
enum InputEvent {
    KeyEvent(Keycode),
    MouseButtonEvent,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Copy, Clone)]
enum InputEffect<Axes, Buttons>
    where Axes: Eq + Ord + Clone,
          Buttons: Eq + Ord + Clone
{
    Axis(Axes, bool),
    Button(Buttons),
}

#[derive(Debug)]
pub struct InputManager<Axes, Buttons>
    where Axes: Eq + Ord + Hash + Clone,
          Buttons: Eq + Ord + Hash + Clone
{
    // Once EnumSet is stable it should be used for these
    // instead of BTreeMap. ♥?
    // Binding of keys to input values.
    bindings: HashMap<InputEvent, InputEffect<Axes, Buttons>>,
    // Input state for axes
    axes: BTreeMap<Axes, f64>,
    // Input states for buttons
    buttons: BTreeMap<Buttons, bool>,
}

use std::collections::hash_map::Entry;

impl<Axes, Buttons> InputManager<Axes, Buttons>
    where Axes: Eq + Ord + Hash + Clone,
          Buttons: Eq + Ord + Hash + Clone
{
    pub fn new() -> Self {
        InputManager {
            bindings: HashMap::new(),
            axes: BTreeMap::new(),
            buttons: BTreeMap::new(),
        }
    }

    pub fn bind_key_to_axis(mut self, keycode: Keycode, axis: Axes, positive: bool) -> Self {

        self.bindings.insert(InputEvent::KeyEvent(keycode),
                             InputEffect::Axis(axis, positive));
        self
    }

    pub fn bind_key_to_button(mut self, keycode: Keycode, button: Buttons) -> Self {
        self.bindings.insert(InputEvent::KeyEvent(keycode), InputEffect::Button(button));
        self
    }


    pub fn update_keydown(&mut self, keycode: Option<Keycode>) {
        if let Some(keycode) = keycode {
            let effect = {
                let e = self.bindings.get(&InputEvent::KeyEvent(keycode));
                match e {
                    None => {
                        return;
                    }
                    Some(effect) => (*effect).clone(),
                }
            };
            self.start_effect(effect);
        }
    }

    pub fn update_keyup(&mut self, keycode: Option<Keycode>) {
        if let Some(keycode) = keycode {
            let effect = {
                let e = self.bindings.get(&InputEvent::KeyEvent(keycode));
                match e {
                    None => {
                        return;
                    }
                    Some(effect) => (*effect).clone(),
                }
            };
            self.end_effect(effect);
        }
    }

    fn start_effect(&mut self, effect: InputEffect<Axes, Buttons>) {}

    fn end_effect(&mut self, effect: InputEffect<Axes, Buttons>) {}

    pub fn mouse_position() {}

    pub fn mouse_scroll_delta() {}

    pub fn get_axis() {}

    pub fn get_axis_raw() {}

    pub fn get_button() {}

    pub fn get_button_down() {}

    pub fn get_button_up() {}

    pub fn get_mouse_button() {}

    pub fn get_mouse_button_down() {}

    pub fn get_mouse_button_up() {}

    pub fn reset_input_axes() {}
}
