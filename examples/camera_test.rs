//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.
// /*
extern crate ggez;
extern crate rand;

use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::graphics::Color;
use ggez::graphics::DrawParam;
use ggez::input::keyboard::KeyCode;
use ggez::mint;
use ggez::{Context, GameResult};

extern crate ggez_goodies;
use ggez_goodies::camera::*;
use nalgebra_glm::Vec2;

struct MainState {
    camera: Camera,
    image: graphics::Image,
    image_location: Vec2,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let camera = Camera::new(WINDOW_WIDTH, WINDOW_HEIGHT, CAMERA_WIDTH, CAMERA_HEIGHT);

        println!("Camera test instructions; WASD move the object, arrow keys move the camera.");
        println!(
            "The red dots are drawn on every integer point in the camera's coordinate \
             system."
        );
        let image =
            graphics::Image::from_color(ctx, 5, 5, Some(graphics::Color::new(1.0, 0.0, 0.0, 1.0)));

        let state = MainState {
            camera,
            image,
            image_location: Vec2::new(0.0, 0.0),
        };
        Ok(state)
    }
}

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

const CAMERA_WIDTH: f32 = 40.0;
const CAMERA_HEIGHT: f32 = 30.0;

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let k_ctx = &ctx.keyboard.clone();
        if k_ctx.is_key_pressed(KeyCode::W) {
            self.image_location.y += 0.1;
        }
        if k_ctx.is_key_pressed(KeyCode::S) {
            self.image_location.y -= 0.1;
        }
        if k_ctx.is_key_pressed(KeyCode::D) {
            self.image_location.x += 0.1;
        }
        if k_ctx.is_key_pressed(KeyCode::A) {
            self.image_location.x -= 0.1;
        }
        if k_ctx.is_key_pressed(KeyCode::Up) {
            self.camera.move_by(Vec2::new(0.0, 0.1));
        }
        if k_ctx.is_key_pressed(KeyCode::Down) {
            self.camera.move_by(Vec2::new(0.0, -0.1));
        }
        if k_ctx.is_key_pressed(KeyCode::Right) {
            self.camera.move_by(Vec2::new(0.1, 0.0));
        }
        if k_ctx.is_key_pressed(KeyCode::Left) {
            self.camera.move_by(Vec2::new(-0.1, 0.0));
        }
        // println!(
        //     "Camera position is now ({}, {}), object position is ({}, {})",
        //     self.camera.location().x,
        //     self.camera.location().y,
        //     self.image_location.x,
        //     self.image_location.y,
        // );
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let half_width = (CAMERA_WIDTH / 2.0) as i32;
        let half_height = (CAMERA_HEIGHT / 2.0) as i32;
        for y in -half_height..half_height {
            for x in -half_width..half_width {
                let frompt = Vec2::new(x as f32, y as f32);
                let (px, py) = self.camera.world_to_screen_coords(frompt);
                let rectangle = ggez::graphics::Rect::new_i32(px, py, 1, 1);
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rectangle,
                    graphics::Color::from((255, 0, 0)),
                )?;
                let dest = DrawParam::new().dest(mint::Point2 { x: 0.0, y: 0.0 });
                canvas.draw(&rectangle, dest);
            }
        }
        self.image
            .draw_camera(&self.camera, &mut canvas, self.image_location, 0.0)?;
        canvas.finish(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("camera_test", "test")
        .window_setup(conf::WindowSetup::default().title("Camera test"))
        .build()?;
    let game = MainState::new(&mut ctx).unwrap();

    event::run(ctx, event_loop, game)
}
