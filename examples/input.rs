//! Simple two-player input example.
//! Control two squares, move them around and change their color.
//!
//! The first square is controlled by the controller:
//! * Use DPad on the controller to move around,
//! * Press South button on the controller to change color,
//!
//! The second square is controller by the keyboard :
//! * Use Arrow Keys to move around,
//! * Press Space on the controller to change color,
//!
//! The third square is controlled by the mouse:
//! * Move the mouse around to move it,
//! * Click left-button to change color
//!
//! Tested with a PS3 controller
//!

extern crate ggez;

use ggez::event::{Axis, Button, MouseButton};
use ggez::graphics::{self, Color};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, GameResult};
use ggez_goodies::input::{InputBinding, InputState, InputStateBuilder};
use ggez_goodies::Point2;
use graphics::{DrawMode, DrawParam, FillOptions, Mesh, Rect};

const NB_PLAYERS: usize = 3;
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum GameAxis {
    Horizontal,
    Vertical,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum GameButton {
    ChangeColor,
}

struct MainState {
    meshes: Vec<graphics::Mesh>,
    colors: Vec<graphics::Color>,
    pos: Vec<Point2>,
    input_state: InputState<GameAxis, GameButton>,
}

impl MainState {
    pub fn new(ctx: &mut Context, input_state: InputState<GameAxis, GameButton>) -> Self {
        let meshes = (0..NB_PLAYERS)
            .map(|_| {
                Mesh::new_rectangle(
                    ctx,
                    DrawMode::Fill(FillOptions::default()),
                    Rect::new(0., 0., 32., 32.),
                    graphics::Color::WHITE,
                )
                .expect("Failed to build mesh")
            })
            .collect();

        let pos = (0..NB_PLAYERS)
            .map(|x| {
                let x = (x + 1) * 64;
                [x as f32, 32.].into()
            })
            .collect();

        let colors = (0..NB_PLAYERS).map(|_| graphics::Color::WHITE).collect();

        MainState {
            meshes,
            pos,
            colors,
            input_state,
        }
    }
}

impl ggez::event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update all player positions
        (0..NB_PLAYERS).for_each(|x| {
            let velocity_x = self.input_state.get_player_axis(GameAxis::Horizontal, x);
            let velocity_y = self.input_state.get_player_axis(GameAxis::Vertical, x);
            self.pos[x].x += velocity_x;
            self.pos[x].y += velocity_y;

            if self
                .input_state
                .get_player_button_pressed(GameButton::ChangeColor, x)
            {
                self.colors[x] = [
                    rand::random::<f32>(),
                    rand::random::<f32>(),
                    rand::random::<f32>(),
                    1.,
                ]
                .into();
            }
        });

        let mouse_pos = self.input_state.get_mouse_position();
        self.pos[2].x = mouse_pos.x;
        self.pos[2].y = mouse_pos.y;

        // Updates the input state
        // Note: you must handle input *before* calling input.update()
        // because update clears the trigger detection flag
        // (for button_pressed and button_released)
        let dt = ctx.time.delta().as_secs_f32();
        self.input_state.update(dt);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        (0..NB_PLAYERS).for_each(|x| {
            canvas.draw(
                &self.meshes[x],
                DrawParam::new().dest(self.pos[x]).color(self.colors[x]),
            )
        });

        canvas.finish(ctx)
    }

    // fn key_down_event(
    //     &mut self,
    //     _ctx: &mut Context,
    //     keycode: KeyCode,
    //     _keymods: KeyMods,
    //     repeat: bool,
    // ) {
    //     if repeat {
    //         return;
    //     }
    //     self.input_state.update_key_down(keycode);
    // }

    // fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
    //     self.input_state.update_key_up(keycode);
    // }

    // fn gamepad_button_down_event(&mut self, ctx: &mut Context, btn: Button, id: GamepadId) {
    //     let id = ctx.gamepad_context.gamepad(id).id();
    //     self.input_state.update_gamepad_down(btn, id.into())
    // }

    // fn gamepad_button_up_event(&mut self, ctx: &mut Context, btn: Button, id: GamepadId) {
    //     let id = ctx.gamepad_context.gamepad(id).id();
    //     self.input_state.update_gamepad_up(btn, id.into())
    // }

    // fn gamepad_axis_event(&mut self, ctx: &mut Context, axis: Axis, value: f32, id: GamepadId) {
    //     let id = ctx.gamepad_context.gamepad(id).id();
    //     self.input_state.update_axis(axis, value, id.into());
    // }

    // fn mouse_button_down_event(
    //     &mut self,
    //     _ctx: &mut Context,
    //     button: MouseButton,
    //     _x: f32,
    //     _y: f32,
    // ) {
    //     self.input_state.update_mouse_button_down(button);
    // }

    // fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
    //     self.input_state.update_mouse_button_up(button);
    // }

    // fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
    //     self.input_state.update_mouse_motion(x, y, dx, dy);
    // }
}

fn main() -> ggez::GameResult<()> {
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("ggez-goodies input", "opinon")
        .build()
        .expect("Failed to build context");

    // Player 0 is controlled using the gamepad
    // Player 1 is controlled using arrow keys on the keyboard
    // Player 2 is controlled by the mouse
    let input_state = InputStateBuilder::new()
        .with_binding(
            InputBinding::new()
                .bind_gamepad_button_to_axis(Button::DPadLeft, GameAxis::Horizontal, false)
                .bind_gamepad_button_to_axis(Button::DPadRight, GameAxis::Horizontal, true)
                .bind_gamepad_button_to_axis(Button::DPadUp, GameAxis::Vertical, false)
                .bind_gamepad_button_to_axis(Button::DPadDown, GameAxis::Vertical, true)
                .bind_gamepad_axis_to_axis(Axis::LeftStickX, GameAxis::Horizontal, false)
                .bind_gamepad_axis_to_axis(Axis::LeftStickY, GameAxis::Vertical, true)
                .bind_gamepad_button_to_button(Button::South, GameButton::ChangeColor),
        )
        .with_binding(
            InputBinding::new()
                .bind_key_to_axis(KeyCode::Left, GameAxis::Horizontal, false)
                .bind_key_to_axis(KeyCode::Right, GameAxis::Horizontal, true)
                .bind_key_to_axis(KeyCode::Up, GameAxis::Vertical, false)
                .bind_key_to_axis(KeyCode::Down, GameAxis::Vertical, true)
                .bind_key_to_button(KeyCode::Space, GameButton::ChangeColor),
        )
        .with_binding(
            InputBinding::new()
                .bind_mouse_button_to_button(MouseButton::Left, GameButton::ChangeColor),
        )
        .build();
    let state = MainState::new(&mut ctx, input_state);
    ggez::event::run(ctx, event_loop, state)
}
