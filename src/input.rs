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
struct AxisStatus {
    position: f64,
    current_direction: f64,
}

impl Default for AxisStatus {
    fn default() -> Self {
        AxisStatus {
            position: 0.0,
            current_direction: 0.0,
        }
    }
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
    axes: BTreeMap<Axes, AxisStatus>,
    // Input states for buttons
    buttons: BTreeMap<Buttons, bool>,
}

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
                if let Some(e) = self.bindings.get(&InputEvent::KeyEvent(keycode)) {
                    e.clone()
                } else {
                    return;
                }
            };
            self.update_effect(effect, true);
        }
    }

    pub fn update_keyup(&mut self, keycode: Option<Keycode>) {
        if let Some(keycode) = keycode {
            let effect = {
                if let Some(e) = self.bindings.get(&InputEvent::KeyEvent(keycode)) {
                    e.clone()
                } else {
                    return;
                }
            };
            self.update_effect(effect, false);
        }
    }

    fn update_effect(&mut self, effect: InputEffect<Axes, Buttons>, started: bool) {
        match effect {
            InputEffect::Axis(axis, direction) => {
                let direction_float = if direction {
                    1.0
                } else {
                    -1.0
                };
                let default_status = AxisStatus {
                    position: 0.0,
                    current_direction: direction_float,
                };
                let axis_status = self.axes.entry(axis).or_insert(default_status);
                axis_status.current_direction = direction_float;
            }
            InputEffect::Button(button) => {
                let button_pressed = self.buttons.entry(button).or_insert(started);
                *button_pressed = started;

            }
        }
    }

    pub fn get_axis(&self, axis: &Axes) -> f64 {
        if let Some(ax) = self.axes.get(axis) {
            ax.position
        } else {
            0.0
        }
    }

    pub fn get_axis_raw() {}

    pub fn get_button(&self, axis: &Buttons) -> bool {
        if let Some(pressed) = self.buttons.get(axis) {
            *pressed
        } else {
            false
        }
    }

    pub fn mouse_position() {}

    pub fn mouse_scroll_delta() {}

    pub fn get_button_down() {}

    pub fn get_button_up() {}

    pub fn get_mouse_button() {}

    pub fn get_mouse_button_down() {}

    pub fn get_mouse_button_up() {}

    pub fn reset_input_axes() {}
}


mod tests {

    #[ignore(unused_imports)]
    use ggez::event::*;
    #[ignore(unused_imports)]
    use super::*;

    #[ignore(dead_code)]
    #[derive(Hash, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
    enum Buttons {
        Up,
        Left,
        Right,
        Down,
    }
    #[test]
    fn test_input_events() {
        let mut im = InputManager::<(), Buttons>::new().bind_key_to_button(Keycode::W, Buttons::Up);

        im.update_keydown(Some(Keycode::W));
        assert!(im.get_button(&Buttons::Up));
        im.update_keyup(Some(Keycode::W));
        assert!(!im.get_button(&Buttons::Up));
    }
}
