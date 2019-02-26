//! An Asteroids-ish example game to show off ggez.
//! The idea is that this game is simple but still
//! non-trivial enough to be interesting.

extern crate ggez;
extern crate rand;

use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::mint::{Point2, Vector2};
use ggez::{Context, GameResult};

extern crate ggez_goodies;
use ggez_goodies::camera::*;

struct MainState {
    camera: Camera,
    image: graphics::Image,
    image_location: Point2<f32>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let camera = Camera::new(WINDOW_WIDTH, WINDOW_HEIGHT, CAMERA_WIDTH, CAMERA_HEIGHT);

        println!("Camera test instructions; WASD move the object, arrow keys move the camera.");
        println!(
            "The red dots are drawn on every integer point in the camera's coordinate \
             system."
        );
        let image = graphics::Image::solid(ctx, 5, graphics::Color::new(1.0, 0.0, 0.0, 1.0))?;

        let state = MainState {
            camera: camera,
            image: image,
            image_location: Point2 { x: 0.0, y: 0.0 },
        };
        Ok(state)
    }
}

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

const CAMERA_WIDTH: f32 = 40.0;
const CAMERA_HEIGHT: f32 = 30.0;

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        let half_width = (CAMERA_WIDTH / 2.0) as i32;
        let half_height = (CAMERA_HEIGHT / 2.0) as i32;
        for y in -half_height..half_height {
            for x in -half_width..half_width {
                let frompt = Point2 {
                    x: x as f32,
                    y: y as f32,
                };
                let (px, py) = self.camera.world_to_screen_coords(frompt);
                let rectangle = ggez::graphics::Rect::new_i32(px, py, 1, 1);
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rectangle,
                    graphics::Color::from((255, 0, 0)),
                )?;
                graphics::draw(ctx, &rectangle, (Point2 { x: 0.0, y: 0.0 },))?;
            }
        }
        self.image
            .draw_camera(&self.camera, ctx, self.image_location, 0.0)?;
        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            event::KeyCode::W => {
                self.image_location.y += 0.1;
            }
            event::KeyCode::A => {
                self.image_location.x -= 0.1;
            }
            event::KeyCode::S => {
                self.image_location.y -= 0.1;
            }
            event::KeyCode::D => {
                self.image_location.x += 0.1;
            }
            event::KeyCode::Up => self.camera.move_by(Vector2 { x: 0.0, y: 0.1 }),
            event::KeyCode::Left => self.camera.move_by(Vector2 { x: -0.1, y: 0.0 }),
            event::KeyCode::Down => self.camera.move_by(Vector2 { x: 0.0, y: -0.1 }),
            event::KeyCode::Right => self.camera.move_by(Vector2 { x: 0.1, y: 0.0 }),
            _ => (),
        };
        println!(
            "Camera position is now ({}, {}), object position is ({}, {})",
            self.camera.location().x,
            self.camera.location().y,
            self.image_location.x,
            self.image_location.y,
        );
    }
}

pub fn main() {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("camera_test", "test")
        .window_setup(conf::WindowSetup::default().title("Camera test"))
        .build()
        .unwrap();
    let game = &mut MainState::new(ctx).unwrap();

    if let Err(e) = event::run(ctx, event_loop, game) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
