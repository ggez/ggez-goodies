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

use std::hash::Hash;
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
    KeyEvent(Keycode), // MouseButtonEvent,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputEffect<Axes, Buttons>
    where Axes: Eq + Hash + Clone,
          Buttons: Eq + Hash + Clone
{
    Axis(Axes, bool),
    Button(Buttons),
}

#[derive(Debug, Copy, Clone)]
struct AxisStatus {
    // Where the axis currently is, in [-1, 1]
    position: f64,
    // Where the axis is moving towards.  Possible
    // values are -1, 0, +1
    // (or a continuous range for analog devices I guess)
    direction: f64,
    // Speed in units per second that the axis
    // moves towards the target value.
    acceleration: f64,
    // Speed in units per second that the axis will
    // fall back toward 0 if the input stops.
    gravity: f64,
}

impl Default for AxisStatus {
    fn default() -> Self {
        AxisStatus {
            position: 0.0,
            direction: 0.0,
            acceleration: 4.0,
            gravity: 3.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct ButtonStatus {
    pressed: bool,
    pressed_last_frame: bool,
}

/// A struct that contains a mapping from physical input events
/// (currently just `Keycode`s) to whatever your logical Axis/Button
/// types are.
pub struct InputBinding<Axes, Buttons>
    where Axes: Hash + Eq + Clone,
          Buttons: Hash + Eq + Clone {
    // Once EnumSet is stable it should be used for these
    // instead of BTreeMap. ♥?
    // Binding of keys to input values.
    bindings: HashMap<InputEvent, InputEffect<Axes, Buttons>>,
}

impl<Axes, Buttons> InputBinding<Axes, Buttons>
    where Axes: Hash + Eq + Clone,
          Buttons: Hash + Eq + Clone {
    
    fn new() -> Self {
        InputBinding {
            bindings: HashMap::new(),
        }
    }

    /// Adds a key binding connecting the given keycode to the given
    /// logical axis.
    pub fn bind_key_to_axis(mut self, keycode: Keycode, axis: Axes, positive: bool) -> Self {
        
        self.bindings.insert(InputEvent::KeyEvent(keycode),
                             InputEffect::Axis(axis.clone(), positive));
        self
    }

    /// Adds a key binding connecting the given keycode to the given
    /// logical button.
    pub fn bind_key_to_button(mut self, keycode: Keycode, button: Buttons) -> Self {
        self.bindings.insert(InputEvent::KeyEvent(keycode),
                             InputEffect::Button(button.clone()));
        self
    }

    /// Takes an physical input type and turns it into a logical input type (keycode -> axis/button).
    pub fn resolve(&self, keycode: Keycode) -> Option<InputEffect<Axes, Buttons>> {
        self.bindings.get(&InputEvent::KeyEvent(keycode)).cloned()
    }
}

#[derive(Debug)]
pub struct InputManager<Axes, Buttons>
    where Axes: Hash + Eq + Clone,
          Buttons: Hash + Eq + Clone
{
    // Input state for axes
    axes: HashMap<Axes, AxisStatus>,
    // Input states for buttons
    buttons: HashMap<Buttons, ButtonStatus>,
}

impl<Axes, Buttons> InputManager<Axes, Buttons>
    where Axes: Eq + Hash + Clone,
          Buttons: Eq + Hash + Clone
{
    pub fn new() -> Self {
        InputManager {
            axes: HashMap::new(),
            buttons: HashMap::new(),
        }
    }

    /// Updates the logical input state based on the actual
    /// physical input state.  Should be called in your update()
    /// handler.
    /// So, it will do things like move the axes and so on.
    pub fn update(&mut self, dt: f64) {
        for (_axis, axis_status) in self.axes.iter_mut() {
            if axis_status.direction != 0.0 {
                // Accelerate the axis towards the
                // input'ed direction.
                let abs_dx = f64::min(axis_status.acceleration * dt,
                                      1.0 - f64::abs(axis_status.position));
                let dx = if axis_status.direction > 0.0 {
                    abs_dx
                } else {
                    -abs_dx
                };
                axis_status.position += dx;
            } else {
                // Gravitate back towards 0.
                let abs_dx = f64::min(axis_status.gravity * dt, f64::abs(axis_status.position));
                let dx = if axis_status.position > 0.0 {
                    -abs_dx
                } else {
                    abs_dx
                };
                axis_status.position += dx;
            }
        }
        for (_button, button_status) in self.buttons.iter_mut() {
            button_status.pressed_last_frame = button_status.pressed;
        }
    }

    /// This method should get called by your key_down_event handler.
    pub fn update_button_down(&mut self, button: Buttons) {
        self.update_effect(InputEffect::Button(button), true);
    }

    /// This method should get called by your key_up_event handler.
    pub fn update_button_up(&mut self, button: Buttons) {
        self.update_effect(InputEffect::Button(button), false);
    }

    /// This method should get called by your key_up_event handler.
    pub fn update_axis_start(&mut self, axis: Axes, positive: bool) {
        self.update_effect(InputEffect::Axis(axis, positive), true);
    }

    pub fn update_axis_stop(&mut self, axis: Axes, positive: bool) {
        self.update_effect(InputEffect::Axis(axis, positive), false);
    }


    /// Takes an InputEffect and actually applies it.
    pub fn update_effect(&mut self, effect: InputEffect<Axes, Buttons>, started: bool) {
        match effect {
            InputEffect::Axis(axis, direction) => {
                let f = || AxisStatus::default();
                let axis_status = self.axes.entry(axis).or_insert_with(f);
                if started {
                    let direction_float = if direction { 1.0 } else { -1.0 };
                    axis_status.direction = direction_float;
                } else {
                    axis_status.direction = 0.0;
                }
            }
            InputEffect::Button(button) => {
                let f = || ButtonStatus::default();
                let button_status = self.buttons.entry(button).or_insert_with(f);
                button_status.pressed = started;
            }
        }
    }

    pub fn get_axis(&mut self, axis: Axes) -> f64 {
        let f = || AxisStatus::default();
        let axis_status = self.axes.entry(axis).or_insert_with(f);
        axis_status.position
    }

    pub fn get_axis_raw(&mut self, axis: Axes) -> f64 {
        let f = || AxisStatus::default();
        let axis_status = self.axes.entry(axis).or_insert_with(f);
        axis_status.direction
    }

    fn get_button(&mut self, button: Buttons) -> ButtonStatus {
        let f = ButtonStatus::default;
        let button_status = self.buttons.entry(button).or_insert_with(f);
        *button_status
    }

    pub fn get_button_down(&mut self, axis: Buttons) -> bool {
        self.get_button(axis).pressed
    }

    pub fn get_button_up(&mut self, axis: Buttons) -> bool {
        !self.get_button(axis).pressed
    }

    /// Returns whether or not the button was pressed this frame,
    /// only returning true if the press happened this frame.
    ///
    /// Basically, `get_button_down()` and `get_button_up()` are level
    /// triggers, this and `get_button_released()` are edge triggered.
    pub fn get_button_pressed(&mut self, axis: Buttons) -> bool {
        let b = self.get_button(axis);
        b.pressed && !b.pressed_last_frame
    }

    pub fn get_button_released(&mut self, axis: Buttons) -> bool {
        let b = self.get_button(axis);
        !b.pressed && b.pressed_last_frame
    }

    pub fn mouse_position() {
        unimplemented!()
    }

    pub fn mouse_scroll_delta() {
        unimplemented!()
    }

    pub fn get_mouse_button() {
        unimplemented!()
    }

    pub fn get_mouse_button_down() {
        unimplemented!()
    }

    pub fn get_mouse_button_up() {
        unimplemented!()
    }

    pub fn reset_input_axes(&mut self) {
        for (_axis, axis_status) in self.axes.iter_mut() {
            axis_status.position = 0.0;
            axis_status.direction = 0.0;
        }
    }
}


#[cfg(test)]
mod tests {
    use ggez::event::*;
    use super::*;

    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    enum Buttons {
        A,
        B,
        Select,
        Start,
    }

    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    enum Axes {
        Horz,
        Vert,
    }

    fn make_input_binding() -> InputBinding<Axes, Buttons> {
        let ib = InputBinding::<Axes, Buttons>::new()
            .bind_key_to_button(Keycode::Z, Buttons::A)
            .bind_key_to_button(Keycode::X, Buttons::B)
            .bind_key_to_button(Keycode::Return, Buttons::Start)
            .bind_key_to_button(Keycode::RShift, Buttons::Select)
            .bind_key_to_button(Keycode::LShift, Buttons::Select)
            .bind_key_to_axis(Keycode::Up, Axes::Vert, true)
            .bind_key_to_axis(Keycode::Down, Axes::Vert, false)
            .bind_key_to_axis(Keycode::Left, Axes::Horz, false)
            .bind_key_to_axis(Keycode::Right, Axes::Horz, true);
        ib
    }
    
    #[test]
    fn test_input_bindings() {
        let ib = make_input_binding();
        assert_eq!(ib.resolve(Keycode::Z), Some(InputEffect::Button(Buttons::A)));
        assert_eq!(ib.resolve(Keycode::X), Some(InputEffect::Button(Buttons::B)));
        assert_eq!(ib.resolve(Keycode::Return), Some(InputEffect::Button(Buttons::Start)));
        assert_eq!(ib.resolve(Keycode::RShift), Some(InputEffect::Button(Buttons::Select)));
        assert_eq!(ib.resolve(Keycode::LShift), Some(InputEffect::Button(Buttons::Select)));

        assert_eq!(ib.resolve(Keycode::Up), Some(InputEffect::Axis(Axes::Vert, true)));
        assert_eq!(ib.resolve(Keycode::Down), Some(InputEffect::Axis(Axes::Vert, false)));
        assert_eq!(ib.resolve(Keycode::Left), Some(InputEffect::Axis(Axes::Horz, false)));
        assert_eq!(ib.resolve(Keycode::Right), Some(InputEffect::Axis(Axes::Horz, true)));

        assert_eq!(ib.resolve(Keycode::Q), None);
        assert_eq!(ib.resolve(Keycode::W), None);
    }
    
    #[test]
    fn test_input_events() {
        let mut im = InputManager::new();
        im.update_button_down(Buttons::A);
        assert!(im.get_button_down(Buttons::A));
        im.update_button_up(Buttons::A);
        assert!(!im.get_button_down(Buttons::A));
        assert!(im.get_button_up(Buttons::A));

        // Push the 'up' button, watch the axis
        // increase to 1.0 but not beyond
        im.update_axis_start(Axes::Vert, true);
        assert!(im.get_axis_raw(Axes::Vert) > 0.0);
        while im.get_axis(Axes::Vert) < 0.99 {
            im.update(0.16);
            assert!(im.get_axis(Axes::Vert) >= 0.0);
            assert!(im.get_axis(Axes::Vert) <= 1.0);
        }
        // Release it, watch it wind down
        im.update_axis_stop(Axes::Vert, true);
        while im.get_axis(Axes::Vert) > 0.01 {
            im.update(0.16);
            assert!(im.get_axis(Axes::Vert) >= 0.0)
        }

        // Do the same with the 'down' button.
        im.update_axis_start(Axes::Vert, false);
        while im.get_axis(Axes::Vert) > -0.99 {
            im.update(0.16);
            assert!(im.get_axis(Axes::Vert) <= 0.0);
            assert!(im.get_axis(Axes::Vert) >= -1.0);
        }
    }

    #[test]
    fn test_button_edge_transitions() {
        let mut im: InputManager<Axis, Buttons> = InputManager::new();

        // Push a key, confirm it's transitioned.
        assert!(!im.get_button_down(Buttons::A));
        im.update_button_down(Buttons::A);
        assert!(im.get_button_down(Buttons::A));
        assert!(im.get_button_pressed(Buttons::A));
        assert!(!im.get_button_released(Buttons::A));

        // Update, confirm it's still down but
        // wasn't pressed this frame
        im.update(0.1);
        assert!(im.get_button_down(Buttons::A));
        assert!(!im.get_button_pressed(Buttons::A));
        assert!(!im.get_button_released(Buttons::A));

        // Release it
        im.update_button_up(Buttons::A);
        assert!(im.get_button_up(Buttons::A));
        assert!(!im.get_button_pressed(Buttons::A));
        assert!(im.get_button_released(Buttons::A));
        im.update(0.1);
        assert!(im.get_button_up(Buttons::A));
        assert!(!im.get_button_pressed(Buttons::A));
        assert!(!im.get_button_released(Buttons::A));
    }
}
