extern crate ggez;
extern crate ggez_goodies;
use ggez::event;
use ggez::graphics;
use ggez::graphics::DrawParam;
use ggez::graphics::Text;
use ggez::winit::event::VirtualKeyCode;
use ggez::Context;
use ggez::GameResult;

use ggez_goodies::scene::*;

struct SharedState {}

struct MainState {
    scenes: SceneStack<SharedState>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut scenes = SceneStack::new(ctx, SharedState {});
        scenes.switch(SceneSwitch::push(StartScene { switch: false }));
        Ok(MainState { scenes })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.scenes.update(ctx);
        self.scenes.input(SceneEvents::None, ctx, true);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.scenes.draw(ctx);
        Ok(())
    }
}

struct StartScene {
    switch: bool,
}

impl Scene<SharedState> for StartScene {
    fn update(&mut self, _: &mut SharedState, ctx: &mut ggez::Context) -> SceneSwitch<SharedState> {
        if ctx.keyboard.is_key_just_pressed(VirtualKeyCode::Space) {
            self.switch = true;
        }
        if self.switch {
            self.switch = false;
            SceneSwitch::replace(Scene1 { switch: false })
        } else {
            SceneSwitch::None
        }
    }

    fn draw(&mut self, _: &mut SharedState, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.draw(
            &Text::new("Press space to switch! Current Scene: ".to_owned() + self.name()),
            DrawParam::default(),
        );

        canvas.finish(ctx)?;

        Ok(())
    }

    fn input(&mut self, _: &mut SharedState, _: SceneEvents, _: &mut ggez::Context, _: bool) {}

    fn name(&self) -> &str {
        "StartScene"
    }
}

#[derive(Clone, Debug)]
struct Scene1 {
    switch: bool,
}

impl Scene<SharedState> for Scene1 {
    fn update(&mut self, _: &mut SharedState, ctx: &mut ggez::Context) -> SceneSwitch<SharedState> {
        if ctx.keyboard.is_key_just_pressed(VirtualKeyCode::Space) {
            self.switch = true;
        }
        if self.switch {
            self.switch = false;
            SceneSwitch::replace(StartScene { switch: false })
        } else {
            SceneSwitch::None
        }
    }

    fn draw(&mut self, _: &mut SharedState, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.draw(
            &Text::new("Press space to switch! Current Scene: ".to_owned() + self.name()),
            DrawParam::default(),
        );

        canvas.finish(ctx)?;

        Ok(())
    }

    fn input(&mut self, _: &mut SharedState, _: SceneEvents, _: &mut ggez::Context, _: bool) {}

    fn name(&self) -> &str {
        "Scene 1"
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("scene_test", "ggez");
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
