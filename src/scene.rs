//! The Scene system is basically for transitioning between
//! *completely* different states that have entirely different game
//! loops and but which all share a state.  It operates as a stack, with new
//! scenes getting pushed to the stack (while the old ones stay in
//! memory unchanged).  Apparently this is basically a push-down automata.
//!
//! Also there's no reason you can't have a Scene contain its own
//! Scene subsystem to do its own indirection.  With a different state
//! type, as well!  What fun!  Though whether you want to go that deep
//! down the rabbit-hole is up to you.  I haven't found it necessary
//! yet.
//!
//! This is basically identical in concept to the Amethyst engine's scene
//! system, the only difference is the details of how the pieces are put
//! together.

use ggez;

pub enum SceneEvents {
    /// No event
    None,
    /// event originated in `mouse_button_down_event()`
    MouseButtonDownEvent,
    /// event originated in `mouse_button_up_event()`
    MouseButtonUpEvent,
    /// event originated in `mouse_motion_event()`
    MouseMotionEvent,
    /// event originated in `raw_mouse_motion_event()`
    RawMouseMotionEvent,
    /// event originated in `mouse_enter_or_leave()`
    MouseEnterOrLeave,
    /// event originated in `mouse_wheel_event()`
    MouseWheelEvent,
    /// event originated in `key_down_event()`
    KeyDownEvent,
    /// event originated in `key_up_event()`
    KeyUpEvent,
    /// event originated in `text_input_event()`
    TextInputEvent,
    /// event originated in `touch_event()`
    TouchEvent,
    /// event originated in `gamepad_button_down_event()`
    GamepadButtonDownEvent,
    /// event originated in `gamepad_button_up_event()`
    GamepadButtonUpEvent,
    /// event originated in `gamepad_axis_event()`
    GamepadAxisEvent,
    /// event originated in `focus_event()`
    FocusEvent,
    /// event originated in `quit_event()`
    QuitEvent,
    /// event originated in `resize_event()`
    ResizeEvent,
}

/// A command to change to a new scene, either by pushing a new one,
/// popping one or replacing the current scene (pop and then push).
pub enum SceneSwitch<S, Ev = SceneEvents, C = ggez::Context> {
    None,
    Push(Box<dyn Scene<S, Ev, C>>),
    Replace(Box<dyn Scene<S, Ev, C>>),
    Pop,
}

/// A trait for you to implement on a scene.
/// Defines the callbacks the scene uses:
/// a common context type `C`, and an input event type.
pub trait Scene<S, Ev = SceneEvents, C = ggez::Context> {
    fn update(&mut self, gameworld: &mut S, ctx: &mut C) -> SceneSwitch<S, Ev, C>;
    fn draw(&mut self, gameworld: &mut S, ctx: &mut C) -> ggez::GameResult<()>;
    fn input(&mut self, gameworld: &mut S, event: Ev, ctx: &mut C, started: bool);
    /// Only used for human-readable convenience (or not at all, tbh)
    fn name(&self) -> &str;
    /// This returns whether or not to draw the next scene down on the
    /// stack as well; this is useful for layers or GUI stuff that
    /// only partially covers the screen.
    fn draw_previous(&self) -> bool {
        false
    }
}

impl<S, Ev, C> SceneSwitch<S, Ev, C> {
    /// Convenient shortcut function for boxing scenes.
    ///
    /// Slightly nicer than writing
    /// `SceneSwitch::Replace(Box::new(x))` all the damn time.
    pub fn replace<N>(scene: N) -> Self
    where
        N: Scene<S, Ev, C> + 'static,
    {
        SceneSwitch::Replace(Box::new(scene))
    }

    /// Same as `replace()` but returns SceneSwitch::Push
    pub fn push<N>(scene: N) -> Self
    where
        N: Scene<S, Ev, C> + 'static,
    {
        SceneSwitch::Push(Box::new(scene))
    }

    /// Shortcut for `SceneSwitch::Pop`.
    ///
    /// Currently a little redundant but multiple pops might be nice.
    pub fn pop() -> Self {
        SceneSwitch::Pop
    }
}

/// A stack of `Scene`'s, together with a context object.
pub struct SceneStack<S, Ev = SceneEvents, C = ggez::Context> {
    pub world: S,
    scenes: Vec<Box<dyn Scene<S, Ev, C>>>,
}

impl<S, Ev, C> SceneStack<S, Ev, C> {
    pub fn new(_ctx: &mut C, global_state: S) -> Self {
        Self {
            world: global_state,
            scenes: Vec::new(),
        }
    }

    /// Add a new scene to the top of the stack.
    pub fn push(&mut self, scene: Box<dyn Scene<S, Ev, C>>) {
        self.scenes.push(scene)
    }

    /// Remove the top scene from the stack and returns it;
    /// panics if there is none.
    pub fn pop(&mut self) -> Box<dyn Scene<S, Ev, C>> {
        self.scenes
            .pop()
            .expect("ERROR: Popped an empty scene stack.")
    }

    /// Returns the current scene; panics if there is none.
    pub fn current(&self) -> &dyn Scene<S, Ev, C> {
        &**self
            .scenes
            .last()
            .expect("ERROR: Tried to get current scene of an empty scene stack.")
    }

    /// Executes the given SceneSwitch command; if it is a pop or replace
    /// it returns `Some(old_scene)`, otherwise `None`
    pub fn switch(
        &mut self,
        next_scene: SceneSwitch<S, Ev, C>,
    ) -> Option<Box<dyn Scene<S, Ev, C>>> {
        match next_scene {
            SceneSwitch::None => None,
            SceneSwitch::Pop => {
                let s = self.pop();
                Some(s)
            }
            SceneSwitch::Push(s) => {
                self.push(s);
                None
            }
            SceneSwitch::Replace(s) => {
                let old_scene = self.pop();
                self.push(s);
                Some(old_scene)
            }
        }
    }

    /// The update function must be on the SceneStack because otherwise
    /// if you try to get the current scene and the world to call
    /// update() on the current scene it causes a double-borrow.  :/
    pub fn update(&mut self, ctx: &mut C) {
        let next_scene = {
            let current_scene = &mut **self
                .scenes
                .last_mut()
                .expect("Tried to update empty scene stack");
            current_scene.update(&mut self.world, ctx)
        };
        self.switch(next_scene);
    }

    /// We walk down the scene stack until we find a scene where we aren't
    /// supposed to draw the previous one, then draw them from the bottom up.
    ///
    /// This allows for layering GUI's and such.
    fn draw_scenes(scenes: &mut [Box<dyn Scene<S, Ev, C>>], world: &mut S, ctx: &mut C) {
        assert!(!scenes.is_empty());
        if let Some((current, rest)) = scenes.split_last_mut() {
            if current.draw_previous() {
                SceneStack::draw_scenes(rest, world, ctx);
            }
            current
                .draw(world, ctx)
                .expect("I would hope drawing a scene never fails!");
        }
    }

    /// Draw the current scene.
    pub fn draw(&mut self, ctx: &mut C) {
        SceneStack::draw_scenes(&mut self.scenes, &mut self.world, ctx)
    }

    /// Feeds the given input event to the current scene.
    pub fn input(&mut self, event: Ev, ctx: &mut C, started: bool) {
        let current_scene = &mut **self
            .scenes
            .last_mut()
            .expect("Tried to do input for empty scene stack");
        current_scene.input(&mut self.world, event, ctx, started);
    }
}
