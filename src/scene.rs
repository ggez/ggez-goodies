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

/// A command to change to a new scene, either by pushign a new one,
/// popping one or replacing the current scene (pop and then push).
pub enum SceneSwitch<C, Ev> {
    None,
    Push(Box<Scene<C, Ev>>),
    Replace(Box<Scene<C, Ev>>),
    Pop,
}

/// A trait for you to implement on a scene.
/// Defines the callbacks the scene uses:
/// a common context type `C`, and an input event type `Ev`.
pub trait Scene<C, Ev> {
    fn update(&mut self, gameworld: &mut C, ctx: &mut ggez::Context) -> SceneSwitch<C, Ev>;
    fn draw(&mut self, gameworld: &mut C, ctx: &mut ggez::Context) -> ggez::GameResult<()>;
    fn input(&mut self, gameworld: &mut C, event: Ev, started: bool);
    /// Only used for human-readable convenience (or not at all, tbh)
    fn name(&self) -> &str;
    /// This returns whether or not to draw the next scene down on the
    /// stack as well; this is useful for layers or GUI stuff that
    /// only partially covers the screen.
    fn draw_previous(&self) -> bool {
        false
    }
}

impl<C, Ev> SceneSwitch<C, Ev> {
    /// Convenient shortcut function for boxing scenes.
    ///
    /// Slightly nicer than writing
    /// `SceneSwitch::Replace(Box::new(x))` all the damn time.
    pub fn replace<S>(scene: S) -> Self
    where
        S: Scene<C, Ev> + 'static,
    {
        SceneSwitch::Replace(Box::new(scene))
    }

    /// Same as `replace()` but returns SceneSwitch::Push
    pub fn push<S>(scene: S) -> Self
    where
        S: Scene<C, Ev> + 'static,
    {
        SceneSwitch::Push(Box::new(scene))
    }
}

/// A stack of `Scene`'s, together with a context object.
pub struct SceneStack<C, Ev> {
    pub world: C,
    scenes: Vec<Box<Scene<C, Ev>>>,
}

impl<C, Ev> SceneStack<C, Ev> {
    pub fn new(ctx: &mut ggez::Context, global_state: C) -> Self {
        Self {
            world: global_state,
            scenes: Vec::new(),
        }
    }

    /// Add a new scene to the top of the stack.
    pub fn push(&mut self, scene: Box<Scene<C, Ev>>) {
        self.scenes.push(scene)
    }

    /// Remove the top scene from the stack and returns it;
    /// panics if there is none.
    pub fn pop(&mut self) -> Box<Scene<C, Ev>> {
        self.scenes
            .pop()
            .expect("ERROR: Popped an empty scene stack.")
    }

    /// Returns the current scene; panics if there is none.
    pub fn current(&self) -> &Scene<C, Ev> {
        &**self.scenes
            .last()
            .expect("ERROR: Tried to get current scene of an empty scene stack.")
    }

    /// Executes the given SceneSwitch command; if it is a pop or replace
    /// it returns `Some(old_scene)`, otherwise `None`
    pub fn switch(&mut self, next_scene: SceneSwitch<C, Ev>) -> Option<Box<Scene<C, Ev>>> {
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

    // These functions must be on the SceneStack because otherwise
    // if you try to get the current scene and the world to call
    // update() on the current scene it causes a double-borrow.  :/
    pub fn update(&mut self, ctx: &mut ggez::Context) {
        let next_scene = {
            let current_scene = &mut **self.scenes
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
    fn draw_scenes(scenes: &mut [Box<Scene<C, Ev>>], world: &mut C, ctx: &mut ggez::Context) {
        assert!(scenes.len() > 0);
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
    pub fn draw(&mut self, ctx: &mut ggez::Context) {
        SceneStack::draw_scenes(&mut self.scenes, &mut self.world, ctx)
    }

    /// Feeds the given input event to the current scene.
    pub fn input(&mut self, event: Ev, started: bool) {
        let current_scene = &mut **self.scenes
            .last_mut()
            .expect("Tried to do input for empty scene stack");
        current_scene.input(&mut self.world, event, started);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Thing {
        scenes: Vec<SceneStack<u32, u32>>,
    }

    #[test]
    fn test1() {
        let x = Thing { scenes: vec![] };
    }
}
