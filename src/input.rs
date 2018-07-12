//! An abstract input state object that gets fed user
//! events and updates itself based on a set of key
//! bindings.
//!
//! The goals are:
//!
//! * Have a layer of abstract key bindings rather than
//! looking at concrete event types
//! * Use this to be able to abstract away differences
//! between keyboards, joysticks and game controllers
//! (rather based on Unity3D),
//! * Do some tweening of input axes and stuff just for
//! fun.
//! * Take ggez's event-based input API, and present event- or
//! state-based API so you can do whichever you want.

// TODO: Handle mice, game pads/joysticks

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

/// The raw ggez input types; the "from" part of an input mapping.
/// 
/// TODO: Desperately needs better name.
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
enum InputType {
    KeyEvent(KeyCode),
}

/// Abstract input values; the "to" part of an input mapping.
/// 
/// This is generic over `Axes` and `Buttons` types; these are
/// types that YOU define.  For instance, for your particular
/// game you may have a "camera" axis, a "movement" axis, and
/// "select", "menu" and "exit" buttons.  You would do something
/// like this:
/// 
/// ```rust
/// use ggez_goodies::input::InputEffect;
/// #[derive(PartialEq, Eq, Hash, Clone)]
/// enum MyAxes {
///     Camera,
///     Movement
/// }
/// #[derive(PartialEq, Eq, Hash, Clone)]
/// enum MyButtons {
///     Select,
///     Menu,
///     Exit
/// }
/// 
/// type MyInputEffect = InputEffect<MyAxes, MyButtons>;
/// ```
/// 
/// TODO: Desperately needs better name.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub enum InputEffect<Axes, Buttons>
where
    Axes: Eq + Hash + Clone,
    Buttons: Eq + Hash + Clone,
{
    Axis(Axes, bool),
    Button(Buttons),
}

/// The stored state of an `Axis`.
/// 
/// An axis is not JUST an exact position, this does
/// some simple linear smoothing of the axis value that
/// is usually quite nice.  This also contains the state
/// and constants necessary for that.
#[derive(Debug, Copy, Clone)]
struct AxisState {
    /// Where the axis currently is, in [-1, 1]
    position: f32,
    /// Where the axis is moving towards.  Possible
    /// values are -1, 0, +1
    /// (or a continuous range for analog devices I guess)
    direction: f32,
    /// Speed in units per second that the axis
    /// moves towards the target value.
    acceleration: f32,
    /// Speed in units per second that the axis will
    /// fall back toward 0 if the input stops.
    gravity: f32,
}

impl Default for AxisState {
    fn default() -> Self {
        AxisState {
            position: 0.0,
            direction: 0.0,
            acceleration: 4.0,
            gravity: 3.0,
        }
    }
}

/// All the state necessary for a button press.
#[derive(Debug, Copy, Clone, Default)]
struct ButtonState {
    pressed: bool,
    pressed_last_frame: bool,
}

/// A struct that contains a mapping from physical input events
/// (currently just `KeyCode`s) to whatever your logical Axis/Button
/// types are.
#[derive(Clone, PartialEq)]
pub struct InputBinding<Axes, Buttons>
where
    Axes: Hash + Eq + Clone,
    Buttons: Hash + Eq + Clone,
{
    // Once EnumSet is stable it should be used for these
    // instead of BTreeMap. ♥?
    // Binding of keys to input values.
    bindings: HashMap<InputType, InputEffect<Axes, Buttons>>,
}

impl<Axes, Buttons> InputBinding<Axes, Buttons>
where
    Axes: Hash + Eq + Clone,
    Buttons: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        InputBinding {
            bindings: HashMap::new(),
        }
    }

    /// Adds a key binding connecting the given keycode to the given
    /// logical axis.
    pub fn bind_key_to_axis(mut self, keycode: KeyCode, axis: Axes, positive: bool) -> Self {
        self.bindings.insert(
            InputType::KeyEvent(keycode),
            InputEffect::Axis(axis.clone(), positive),
        );
        self
    }

    /// Adds a key binding connecting the given keycode to the given
    /// logical button.
    pub fn bind_key_to_button(mut self, keycode: KeyCode, button: Buttons) -> Self {
        self.bindings.insert(
            InputType::KeyEvent(keycode),
            InputEffect::Button(button.clone()),
        );
        self
    }

    /// Takes an physical input type and turns it into a logical input type (keycode -> axis/button).
    pub fn resolve(&self, keycode: KeyCode) -> Option<InputEffect<Axes, Buttons>> {
        self.bindings.get(&InputType::KeyEvent(keycode)).cloned()
    }
}

/// The object that tracks the current state of the input controls,
/// such as axes, bindings, etc.
#[derive(Debug, Clone)]
pub struct InputState<Axes, Buttons>
where
    Axes: Hash + Eq + Clone,
    Buttons: Hash + Eq + Clone,
{
    // Input state for axes
    axes: HashMap<Axes, AxisState>,
    // Input states for buttons
    buttons: HashMap<Buttons, ButtonState>,
}

impl<Axes, Buttons> InputState<Axes, Buttons>
where
    Axes: Eq + Hash + Clone,
    Buttons: Eq + Hash + Clone,
{

    pub fn new() -> Self {
        Self {
            axes: HashMap::default(),
            buttons: HashMap::default(),
        }
    }

    /// Updates the logical input state based on the actual
    /// physical input state.  Should be called in your update()
    /// handler.  It will do things like move the axes and so on.
    pub fn update(&mut self, dt: f32) {
        for (_axis, axis_status) in self.axes.iter_mut() {
            if axis_status.direction != 0.0 {
                // Accelerate the axis towards the
                // input'ed direction.
                let vel = axis_status.acceleration * dt;
                let pending_position = axis_status.position + if axis_status.direction > 0.0 {
                    vel
                } else {
                    -vel
                };
                axis_status.position = if pending_position > 1.0 {
                    1.0
                } else if pending_position < -1.0 {
                    -1.0
                } else {
                    pending_position
                }
            } else {
                // Gravitate back towards 0.
                let abs_dx = f32::min(axis_status.gravity * dt, f32::abs(axis_status.position));
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

    /// This method should get called by your `key_down_event` handler.
    pub fn update_button_down(&mut self, button: Buttons) {
        self.update_effect(InputEffect::Button(button), true);
    }

    /// This method should get called by your `key_up_event` handler.
    pub fn update_button_up(&mut self, button: Buttons) {
        self.update_effect(InputEffect::Button(button), false);
    }

    /// This method should get called by your `key_down_event` handler.
    pub fn update_axis_start(&mut self, axis: Axes, positive: bool) {
        self.update_effect(InputEffect::Axis(axis, positive), true);
    }

    /// This method should get called by your `key_up_event` handler.
    pub fn update_axis_stop(&mut self, axis: Axes, positive: bool) {
        self.update_effect(InputEffect::Axis(axis, positive), false);
    }

    /// Takes an InputEffect and actually applies it.
    pub fn update_effect(&mut self, effect: InputEffect<Axes, Buttons>, started: bool) {
        match effect {
            InputEffect::Axis(axis, positive) => {
                let f = || AxisState::default();
                let axis_status = self.axes.entry(axis).or_insert_with(f);
                if started {
                    let direction_float = if positive { 1.0 } else { -1.0 };
                    axis_status.direction = direction_float;
                } else if (positive && axis_status.direction > 0.0)
                    || (!positive && axis_status.direction < 0.0)
                {
                    axis_status.direction = 0.0;
                }
            }
            InputEffect::Button(button) => {
                let f = || ButtonState::default();
                let button_status = self.buttons.entry(button).or_insert_with(f);
                button_status.pressed = started;
            }
        }
    }

    pub fn get_axis(&self, axis: Axes) -> f32 {
        let d = AxisState::default();
        let axis_status = self.axes.get(&axis).unwrap_or(&d);
        axis_status.position
    }

    pub fn get_axis_raw(&self, axis: Axes) -> f32 {
        let d = AxisState::default();
        let axis_status = self.axes.get(&axis).unwrap_or(&d);
        axis_status.direction
    }

    fn get_button(&self, button: Buttons) -> ButtonState {
        let d = ButtonState::default();
        let button_status = self.buttons.get(&button).unwrap_or(&d);
        *button_status
    }

    pub fn get_button_down(&self, axis: Buttons) -> bool {
        self.get_button(axis).pressed
    }

    pub fn get_button_up(&self, axis: Buttons) -> bool {
        !self.get_button(axis).pressed
    }

    /// Returns whether or not the button was pressed this frame,
    /// only returning true if the press happened this frame.
    ///
    /// Basically, `get_button_down()` and `get_button_up()` are level
    /// triggers, this and `get_button_released()` are edge triggered.
    pub fn get_button_pressed(&self, axis: Buttons) -> bool {
        let b = self.get_button(axis);
        b.pressed && !b.pressed_last_frame
    }

    pub fn get_button_released(&self, axis: Buttons) -> bool {
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

    pub fn reset_input_state(&mut self) {
        for (_axis, axis_status) in self.axes.iter_mut() {
            axis_status.position = 0.0;
            axis_status.direction = 0.0;
        }

        for (_button, button_status) in self.buttons.iter_mut() {
            button_status.pressed = false;
            button_status.pressed_last_frame = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::default::Default;
    use ggez::event::{KeyCode};
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
            .bind_key_to_button(KeyCode::Z, Buttons::A)
            .bind_key_to_button(KeyCode::X, Buttons::B)
            .bind_key_to_button(KeyCode::Return, Buttons::Start)
            .bind_key_to_button(KeyCode::RShift, Buttons::Select)
            .bind_key_to_button(KeyCode::LShift, Buttons::Select)
            .bind_key_to_axis(KeyCode::Up, Axes::Vert, true)
            .bind_key_to_axis(KeyCode::Down, Axes::Vert, false)
            .bind_key_to_axis(KeyCode::Left, Axes::Horz, false)
            .bind_key_to_axis(KeyCode::Right, Axes::Horz, true);
        ib
    }

    #[test]
    fn test_input_bindings() {
        let ib = make_input_binding();
        assert_eq!(
            ib.resolve(KeyCode::Z),
            Some(InputEffect::Button(Buttons::A))
        );
        assert_eq!(
            ib.resolve(KeyCode::X),
            Some(InputEffect::Button(Buttons::B))
        );
        assert_eq!(
            ib.resolve(KeyCode::Return),
            Some(InputEffect::Button(Buttons::Start))
        );
        assert_eq!(
            ib.resolve(KeyCode::RShift),
            Some(InputEffect::Button(Buttons::Select))
        );
        assert_eq!(
            ib.resolve(KeyCode::LShift),
            Some(InputEffect::Button(Buttons::Select))
        );

        assert_eq!(
            ib.resolve(KeyCode::Up),
            Some(InputEffect::Axis(Axes::Vert, true))
        );
        assert_eq!(
            ib.resolve(KeyCode::Down),
            Some(InputEffect::Axis(Axes::Vert, false))
        );
        assert_eq!(
            ib.resolve(KeyCode::Left),
            Some(InputEffect::Axis(Axes::Horz, false))
        );
        assert_eq!(
            ib.resolve(KeyCode::Right),
            Some(InputEffect::Axis(Axes::Horz, true))
        );

        assert_eq!(ib.resolve(KeyCode::Q), None);
        assert_eq!(ib.resolve(KeyCode::W), None);
    }

    #[test]
    fn test_input_events() {
        let mut im = InputState::new();
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

        // Test the transition from 'up' to 'down'
        im.update_axis_start(Axes::Vert, true);
        while im.get_axis(Axes::Vert) < 1.0 {
            im.update(0.16);
        }
        im.update_axis_start(Axes::Vert, false);
        im.update(0.16);
        assert!(im.get_axis(Axes::Vert) < 1.0);
        im.update_axis_stop(Axes::Vert, true);
        assert!(im.get_axis_raw(Axes::Vert) < 0.0);
        im.update_axis_stop(Axes::Vert, false);
        assert_eq!(im.get_axis_raw(Axes::Vert), 0.0);
    }

    #[test]
    fn test_button_edge_transitions() {
        let mut im: InputState<Axes, Buttons> = InputState::new();

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
