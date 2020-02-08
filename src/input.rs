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
//!
//! In this module:
//! * "physical" means Hardware button
//! * "logical" means User-defined button
//! * "raw" means unaffected by tweening on input axes
//!
//!
//! TODO: Handle mouse, joysticks
//! Joysticks will probably be a pain because gilrs (and hence ggez)
//! returns their values as f32, which does not implement Hash or Eq, 
//! making them unusable as keys for HashMaps.  

use ggez::event::{Button, KeyCode};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

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
    KeyEvent(KeyCode),    // MouseButtonEvent,
    GamepadEvent(Button), // Gamepad Event
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
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
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
#[derive(Debug, Copy, Clone, PartialEq)]
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
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct ButtonState {
    pressed: bool,
    pressed_last_frame: bool,
}

impl ButtonState {
    /// Is the button pressed or not
    pub fn pressed(&self) -> bool {
        self.pressed
    }

    /// Trigger detector for the button
    pub fn pressed_last_frame(&self) -> bool {
        self.pressed_last_frame
    }
}

/// A struct that contains a mapping from physical input events
/// (currently just `KeyCode`s) to whatever your logical Axis/Button
/// types are.
#[derive(Default, Debug, Eq, PartialEq, Clone)]
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
    /// The default constructor for an InputBinding. Same as calling InputBinding::default().
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
            InputEffect::Axis(axis, positive),
        );
        self
    }

    /// Adds a key binding connecting the given keycode to the given
    /// logical button.
    pub fn bind_key_to_button(mut self, keycode: KeyCode, button: Buttons) -> Self {
        self.bindings
            .insert(InputType::KeyEvent(keycode), InputEffect::Button(button));
        self
    }

    /// Adds a gamepad binding connecting the given Gamepad Button to the given
    /// logical axis.
    pub fn bind_gamepad_button_to_axis(
        mut self,
        button: Button,
        axis: Axes,
        positive: bool,
    ) -> Self {
        self.bindings.insert(
            InputType::GamepadEvent(button),
            InputEffect::Axis(axis, positive),
        );
        self
    }

    /// Adds a gamepad binding connecting the given Gamepad Button to the given
    /// logical button.
    pub fn bind_gamepad_button_to_button(mut self, ggez_button: Button, button: Buttons) -> Self {
        self.bindings.insert(
            InputType::GamepadEvent(ggez_button),
            InputEffect::Button(button),
        );
        self
    }

    /// Takes an physical input type and turns it into a logical input type (keycode -> axis/button).
    pub fn resolve(&self, keycode: KeyCode) -> Option<InputEffect<Axes, Buttons>> {
        self.bindings.get(&InputType::KeyEvent(keycode)).cloned()
    }

    /// Takes a physical Gamepad input type and turns it into a logical input type (gamepad::button -> axis/button).
    pub fn resolve_gamepad(&self, button: Button) -> Option<InputEffect<Axes, Buttons>> {
        self.bindings.get(&InputType::GamepadEvent(button)).cloned()
    }
}

/// The object that tracks the current state of the input controls,
/// such as axes, bindings, etc.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlayerInputState<Axes, Buttons>
where
    Axes: Hash + Eq + Clone,
    Buttons: Hash + Eq + Clone,
{
    /// Input state for axes
    axes: HashMap<Axes, AxisState>,
    /// Input states for buttons
    buttons: HashMap<Buttons, ButtonState>,
}

impl<Axes, Buttons> Default for PlayerInputState<Axes, Buttons>
where
    Axes: Hash + Eq + Clone,
    Buttons: Hash + Eq + Clone,
{
    fn default() -> Self {
        PlayerInputState::new()
    }
}

impl<Axes, Buttons> PlayerInputState<Axes, Buttons>
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
                let pending_position = axis_status.position
                    + if axis_status.direction > 0.0 {
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

    /// Used for testing purposes
    #[allow(dead_code)]
    pub fn update_button_down(&mut self, button: Buttons) {
        self.update_effect(InputEffect::Button(button), true);
    }

    /// Used for testing purposes
    #[allow(dead_code)]
    pub fn update_button_up(&mut self, button: Buttons) {
        self.update_effect(InputEffect::Button(button), false);
    }

    /// Used for testing purposes
    #[allow(dead_code)]
    pub fn update_axis_start(&mut self, axis: Axes, positive: bool) {
        self.update_effect(InputEffect::Axis(axis, positive), true);
    }

    /// Used for testing purposes
    #[allow(dead_code)]
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

    /// Gets the value of a logical Axis
    pub fn get_axis(&self, axis: Axes) -> f32 {
        let d = AxisState::default();
        let axis_status = self.axes.get(&axis).unwrap_or(&d);
        axis_status.position
    }

    /// Gets the raw value of a logical Axis
    pub fn get_axis_raw(&self, axis: Axes) -> f32 {
        let d = AxisState::default();
        let axis_status = self.axes.get(&axis).unwrap_or(&d);
        axis_status.direction
    }

    /// Gets the state of a logical Button
    fn get_button(&self, button: Buttons) -> ButtonState {
        let d = ButtonState::default();
        let button_status = self.buttons.get(&button).unwrap_or(&d);
        *button_status
    }

    pub fn get_button_down(&self, axis: Buttons) -> bool {
        self.get_button(axis).pressed
    }

    /// Used for testing purposes
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn mouse_position() {
        unimplemented!()
    }

    #[allow(dead_code)]
    pub fn mouse_scroll_delta() {
        unimplemented!()
    }

    #[allow(dead_code)]
    pub fn get_mouse_button() {
        unimplemented!()
    }

    #[allow(dead_code)]
    pub fn get_mouse_button_down() {
        unimplemented!()
    }

    #[allow(dead_code)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct InputState<Axes, Buttons>
where
    Axes: Hash + Eq + Clone,
    Buttons: Hash + Eq + Clone,
{
    input_bindings: HashMap<usize, InputBinding<Axes, Buttons>>,
    player_states: HashMap<usize, PlayerInputState<Axes, Buttons>>,
}

impl<Axes, Buttons> Default for InputState<Axes, Buttons>
where
    Axes: Hash + Eq + Clone + Debug,
    Buttons: Hash + Eq + Clone + Debug,
{
    fn default() -> Self {
        InputState::new()
    }
}

/// The ID of the default player
const DEFAULT_PLAYER: usize = 0;

impl<Axes, Buttons> InputState<Axes, Buttons>
where
    Axes: Hash + Eq + Clone + Debug,
    Buttons: Hash + Eq + Clone + Debug,
{
    /// Default constructor for an empty InputState
    fn new() -> Self {
        InputState {
            input_bindings: HashMap::default(),
            player_states: HashMap::default(),
        }
    }

    /// Updates all players state
    pub fn update(&mut self, dt: f32) {
        self.player_states.values_mut().for_each(|ps| ps.update(dt))
    }

    /// Signals to all player state that a key was pressed, updating them accordingly
    pub fn update_key_down(&mut self, key: KeyCode) {
        self.update_key(key, true)
    }

    /// Signals to all player state that a key was released, updating them accordingly
    pub fn update_key_up(&mut self, key: KeyCode) {
        self.update_key(key, false)
    }

    /// Code reuse logic for update_key_down & update_key_up
    /// Effectively signals the states that a key was pressed or released
    fn update_key(&mut self, key: KeyCode, started: bool) {
        for (player_id, binding) in self.input_bindings.iter() {
            if let Some(effect) = binding.resolve(key) {
                let is = self.player_states.entry(*player_id).or_default();
                is.update_effect(effect, started);
            }
        }
    }

    /// Signals the target player's state that a physical Gamepad Button was pressed
    pub fn update_gamepad_down(&mut self, gp_button: Button, player_id: usize) {
        self.update_gamepad(gp_button, player_id, true)
    }

    /// Signals the target player's state that a physical Gamepad Button was released
    pub fn update_gamepad_up(&mut self, gp_button: Button, player_id: usize) {
        self.update_gamepad(gp_button, player_id, false)
    }

    /// Code reuse logic for update_gamepad_down & update_gamepad_up
    /// Effectively signals the target player's state that a gamepad button
    /// was pressed or released
    fn update_gamepad(&mut self, gp_button: Button, player_id: usize, started: bool) {
        if let Some(effect) = self
            .input_bindings
            .get_mut(&player_id)
            .and_then(|ib| ib.resolve_gamepad(gp_button))
        {
            let is = self.player_states.entry(player_id).or_default();
            is.update_effect(effect, started);
        }
    }

    /// Gets the value of a logical axis for the target player
    pub fn get_player_axis(&self, axis: Axes, player_id: usize) -> f32 {
        self.player_states
            .get(&player_id)
            .map(|ps| ps.get_axis(axis))
            .unwrap_or(0.)
    }

    /// Gets the value of a logical axis for the default player
    pub fn get_default_player_axis(&self, axis: Axes) -> f32 {
        self.get_player_axis(axis, DEFAULT_PLAYER)
    }

    /// Gets the raw value of a logical axis for the target player.
    /// The raw value is the exact value of an axis at the present moment, not affected
    /// by the easing factor.
    pub fn get_player_axis_raw(&self, axis: Axes, player_id: usize) -> f32 {
        self.player_states
            .get(&player_id)
            .map(|ps| ps.get_axis_raw(axis))
            .unwrap_or(0.)
    }

    /// Gets the raw value of a logical axis for the default player.
    /// The raw value is the exact value of an axis at the present moment, not affected
    /// by the easing factor.
    pub fn get_default_player_axis_raw(&self, axis: Axes) -> f32 {
        self.get_player_axis_raw(axis, DEFAULT_PLAYER)
    }

    /// Gets the state of a logical button for the target player
    pub fn get_player_button(&self, button: Buttons, player_id: usize) -> ButtonState {
        self.player_states
            .get(&player_id)
            .map(|ps| ps.get_button(button))
            .unwrap_or_default()
    }

    /// Gets the state of a logical button for the default player
    pub fn get_default_player_button(&self, button: Buttons) -> ButtonState {
        self.get_player_button(button, DEFAULT_PLAYER)
    }

    /// Asks if the logical button is held down for the target player
    pub fn get_player_button_down(&self, button: Buttons, player_id: usize) -> bool {
        self.player_states
            .get(&player_id)
            .map(|ps| ps.get_button_down(button))
            .unwrap_or(false)
    }

    /// Asks if the logical button is held down for the default player
    pub fn get_default_player_button_down(&self, button: Buttons) -> bool {
        self.get_player_button_down(button, DEFAULT_PLAYER)
    }

    /// Asks if the logical button for the target player has just started
    /// being pressed during the last update
    pub fn get_player_button_pressed(&self, button: Buttons, player_id: usize) -> bool {
        self.player_states
            .get(&player_id)
            .map(|ps| ps.get_button_pressed(button))
            .unwrap_or(false)
    }

    /// Asks if the logical button for the default player has just started
    /// being pressed during the last update
    pub fn get_default_player_button_pressed(&self, button: Buttons) -> bool {
        self.get_player_button_pressed(button, DEFAULT_PLAYER)
    }

    /// Asks if the logical button for the target player has just
    /// been released during the last update
    pub fn get_player_button_released(&self, button: Buttons, player_id: usize) -> bool {
        self.player_states
            .get(&player_id)
            .map(|ps| ps.get_button_released(button))
            .unwrap_or(false)
    }

    /// Asks if the logical button for the default player has just  
    /// been released during the last update
    pub fn get_default_player_button_released(&self, button: Buttons) -> bool {
        self.get_player_button_pressed(button, DEFAULT_PLAYER)
    }

    /// Resets the Input State for a given player
    pub fn reset_player_input_state(&mut self, player_id: usize) {
        if let Some(input_state) = self.player_states.get_mut(&player_id) {
            input_state.reset_input_state()
        }
    }

    /// Resets the Input State for the default player
    pub fn reset_default_player_input_state(&mut self) {
        if let Some(input_state) = self.player_states.get_mut(&DEFAULT_PLAYER) {
            input_state.reset_input_state()
        }
    }

    /// Resets all players Input State
    pub fn reset_all_player_input_state(&mut self) {
        self.player_states
            .values_mut()
            .for_each(|ps| ps.reset_input_state())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Builder pattern wrapping an InputState.
///
/// This can be used to create an InputState parametrized with bindings.
/// For example, building an input state for a platformer game might look like this :
///
/// ```rust
/// use ggez_goodies::input::{InputBinding, InputStateBuilder};
/// use ggez::event::KeyCode;
///
/// #[derive(Debug, Hash, PartialEq, Eq, Clone)]
/// enum MyAxes { Horizontal, Vertical }
///
/// #[derive(Debug, Hash, PartialEq, Eq, Clone)]
/// enum MyButtons { Jump }
///
/// let input_state =
///     InputStateBuilder::new()
///     .with_binding(
///         InputBinding::new()
///             .bind_key_to_axis(KeyCode::Left, MyAxes::Horizontal, false) // Left is negative X    
///             .bind_key_to_axis(KeyCode::Right, MyAxes::Horizontal, true) // Right is negative X
///             .bind_key_to_axis(KeyCode::Up, MyAxes::Vertical, false)     // (in ggez's coordinate system) Up is negative Y
///             .bind_key_to_axis(KeyCode::Down, MyAxes::Vertical, true)    // (in ggez's coordinate system) Down is positive Y
///             .bind_key_to_button(KeyCode::Space, MyButtons::Jump)      // Space will make the player jump
///     )
///     .build();
/// ```
pub struct InputStateBuilder<Axes, Buttons>
where
    Axes: Hash + Eq + Clone,
    Buttons: Hash + Eq + Clone,
{
    curr_player_id: usize,
    bindings: HashMap<usize, InputBinding<Axes, Buttons>>,
}

impl<Axes, Buttons> Default for InputStateBuilder<Axes, Buttons>
where
    Axes: Hash + Eq + Clone,
    Buttons: Hash + Eq + Clone,
{
    fn default() -> Self {
        InputStateBuilder::new()
    }
}

impl<Axes, Buttons> InputStateBuilder<Axes, Buttons>
where
    Axes: Hash + Eq + Clone,
    Buttons: Hash + Eq + Clone,
{
    /// Default constructor for the InputStateBuilder.
    /// Same as calling `default`.
    pub fn new() -> Self {
        InputStateBuilder {
            curr_player_id: 0,
            bindings: HashMap::default(),
        }
    }

    /// Adds a binding to the input state being built
    /// This binding will be used for the next unused player ID.
    /// (on the first call, it will be 0)
    pub fn with_binding(mut self, binding: InputBinding<Axes, Buttons>) -> Self {
        // Because we allow inserting a binding for player X, we need to find
        // an unused ID to make sure we do not accidentally override an
        // existing InputBinding already set up by the user
        let next_unused_id = (self.curr_player_id..)
            .find(|id| self.bindings.get(id).is_none())
            .unwrap();
        self.curr_player_id = next_unused_id;
        self.with_player_binding(binding, next_unused_id)
    }

    /// Adds a binding to the input state being built
    /// This binding will be used for the specified player
    pub fn with_player_binding(
        mut self,
        binding: InputBinding<Axes, Buttons>,
        player_id: usize,
    ) -> Self {
        self.bindings.insert(player_id, binding);
        self
    }

    /// Builds the InputState that has been parametrized
    /// by this builder
    pub fn build(self) -> InputState<Axes, Buttons> {
        InputState {
            player_states: HashMap::default(),
            input_bindings: self.bindings,
        }
    }
}

#[cfg(test)]
mod tests {
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

    fn make_input_binding_multiple() -> (InputBinding<Axes, Buttons>, InputBinding<Axes, Buttons>) {
        (
            InputBinding::<Axes, Buttons>::new()
                .bind_key_to_axis(KeyCode::Up, Axes::Vert, false)
                .bind_key_to_axis(KeyCode::Down, Axes::Vert, true)
                .bind_key_to_axis(KeyCode::Left, Axes::Horz, false)
                .bind_key_to_axis(KeyCode::Right, Axes::Horz, true)
                .bind_key_to_button(KeyCode::Q, Buttons::A)
                .bind_key_to_button(KeyCode::E, Buttons::B),
            InputBinding::<Axes, Buttons>::new()
                .bind_gamepad_button_to_axis(Button::DPadUp, Axes::Vert, false)
                .bind_gamepad_button_to_axis(Button::DPadDown, Axes::Vert, true)
                .bind_gamepad_button_to_axis(Button::DPadLeft, Axes::Horz, false)
                .bind_gamepad_button_to_axis(Button::DPadRight, Axes::Horz, true)
                .bind_gamepad_button_to_button(Button::East, Buttons::A)
                .bind_gamepad_button_to_button(Button::South, Buttons::B),
        )
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
    /// Tests that events are being correctly dispatched
    /// Inner behaviour of PlayerInputState is verified by other tests
    fn test_input_bindings_multiple() {
        let (player_one, player_two) = make_input_binding_multiple();
        let mut input_state = InputStateBuilder::new()
            .with_binding(player_one)
            .with_binding(player_two)
            .build();

        input_state.update_key_down(KeyCode::Up);
        assert!(input_state.get_player_axis_raw(Axes::Vert, 0) < 0.);

        input_state.update_key_up(KeyCode::Up);
        input_state.update_key_down(KeyCode::Down);
        assert!(input_state.get_player_axis_raw(Axes::Vert, 0) > 0.);

        input_state.update_key_up(KeyCode::Down);
        assert_eq!(input_state.get_player_axis_raw(Axes::Vert, 0), 0.);
        
        input_state.update_gamepad_down(Button::DPadLeft, 1);
        assert!(input_state.get_player_axis_raw(Axes::Horz, 1) < 0.);
        
        input_state.update_gamepad_up(Button::DPadLeft, 1);
        input_state.update_gamepad_down(Button::DPadRight, 1);
        assert!(input_state.get_player_axis_raw(Axes::Horz, 1) > 0.);
        
        input_state.update_gamepad_up(Button::DPadRight, 1);
        assert_eq!(input_state.get_player_axis_raw(Axes::Horz, 1), 0.);

        input_state.update_gamepad_down(Button::East, 1);
        assert!(input_state.get_player_button_pressed(Buttons::A, 1));
        assert!(input_state.get_player_button_down(Buttons::A, 1));
        input_state.update(0.1);
        assert!(!input_state.get_player_button_pressed(Buttons::A, 1));
        assert!(input_state.get_player_button_down(Buttons::A, 1));

        input_state.update_gamepad_up(Button::East, 1);
        input_state.update_gamepad_down(Button::South, 1);
        assert!(input_state.get_player_button_pressed(Buttons::B, 1));
    }

    #[test]
    fn test_input_events() {
        let mut im = PlayerInputState::new();
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
        let mut im: PlayerInputState<Axes, Buttons> = PlayerInputState::new();

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
