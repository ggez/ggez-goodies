//! Simple two-player input example.
//! Control two squares, move them around and change their color.
//!
//! The left square is controlled by the controller: 
//! * Use DPad on the controller to move around,
//! * Press South button on the controller to change color,
//!
//! The right square is controller by the keyboard : 
//! * Use Arrow Keys to move around,
//! * Press Space on the controller to change color,
//!
//! Tested with a PS3 controller
//!

extern crate ggez;

use ggez::graphics;
use ggez::{Context, GameResult};
use ggez::event::{KeyCode, KeyMods, Button};
use ggez::input::gamepad::GamepadId;
use ggez_goodies::Point2;
use ggez_goodies::input::{InputState, InputBinding, InputStateBuilder};
use graphics::{DrawParam, DrawMode, FillOptions, Mesh, Rect};

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
        let meshes = (0..2).map(|_| {
            Mesh::new_rectangle(
                ctx,
                DrawMode::Fill(FillOptions::default()),
                Rect::new(0.,0.,32.,32.),
                graphics::WHITE,
            )
            .expect("Failed to build mesh")
        }).collect();

        let pos = (0..2).map(|x| {
            let x = (x + 1) * 64;
            [x as f32, 32.].into()
        }).collect(); 

        MainState {
            meshes,
            pos,
            colors: vec![graphics::WHITE, graphics::WHITE],
            input_state,
        }
    }
}

impl ggez::event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update all player positions
        (0..2)
        .for_each(|x: usize| {
            let velocity_x = self.input_state.get_player_axis(GameAxis::Horizontal, x);
            let velocity_y = self.input_state.get_player_axis(GameAxis::Vertical, x);
            self.pos[x].x += velocity_x; 
            self.pos[x].y += velocity_y; 
            
            if self.input_state.get_player_button_pressed(GameButton::ChangeColor, x) {
                self.colors[x] = [rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>(), 1.].into();
            }
        });
        
        // Updates the input state
        // Note: you must handle input *before* calling input.update() 
        // because update clears the trigger detection flag 
        // (for button_pressed and button_released)
        let dt = ggez::timer::delta(ctx).as_secs_f32();
        self.input_state.update(dt);
        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK); 

        (0..2)
        .map(|x: usize| {
            graphics::draw(
                ctx, 
                &self.meshes[x], 
                DrawParam::default().dest(self.pos[x].clone()).color(self.colors[x]),
            )
        })
        .collect::<GameResult<()>>()?;
        
        graphics::present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, repeat: bool) {
        if repeat {
            return;
        }
        self.input_state.update_key_down(keycode);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        self.input_state.update_key_up(keycode);
    }

    fn gamepad_button_down_event(&mut self, ctx: &mut Context, btn: Button, id: GamepadId) {
        let id = ctx.gamepad_context.gamepad(id).id();
        self.input_state.update_gamepad_down(btn, id.into())
    }
    
    fn gamepad_button_up_event(&mut self, ctx: &mut Context, btn: Button, id: GamepadId) {
        let id = ctx.gamepad_context.gamepad(id).id();
        self.input_state.update_gamepad_up(btn, id.into())
    }
}

fn main() -> ggez::GameResult<()> {
    let (mut ctx, mut event_loop) = ggez::ContextBuilder::new("ggez-goodies input", "opinon")
        .build()
        .expect("Failed to build context");

    // Player 0 is controlled using the gamepad
    // Player 1 is controlled using arrow keys on the keyboard
    let input_state = InputStateBuilder::new()
    .with_binding(
        InputBinding::new()
            .bind_gamepad_button_to_axis(Button::DPadLeft, GameAxis::Horizontal, false)
            .bind_gamepad_button_to_axis(Button::DPadRight, GameAxis::Horizontal, true)
            .bind_gamepad_button_to_axis(Button::DPadUp, GameAxis::Vertical, false)
            .bind_gamepad_button_to_axis(Button::DPadDown, GameAxis::Vertical, true)
            .bind_gamepad_button_to_button(Button::South, GameButton::ChangeColor)
        )
        .with_binding(
            InputBinding::new()
                .bind_key_to_axis(KeyCode::Left, GameAxis::Horizontal, false)
                .bind_key_to_axis(KeyCode::Right, GameAxis::Horizontal, true)
                .bind_key_to_axis(KeyCode::Up, GameAxis::Vertical, false)
                .bind_key_to_axis(KeyCode::Down, GameAxis::Vertical, true)
                .bind_key_to_button(KeyCode::Space, GameButton::ChangeColor)
        )
        .build();
    let mut state = MainState::new(&mut ctx, input_state);
    ggez::event::run(&mut ctx, &mut event_loop, &mut state)
}
