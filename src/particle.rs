extern crate nalgebra as na;

use ggez::{GameResult, Context};
use ggez::graphics;

type Point2 = na::Point2<f64>;
type Vector2 = na::Vector2<f64>;

struct Particle {
    pos: Point2,
    vel: Vector2,
}

impl Particle {
    fn new(pos: Point2, vel: Vector2) -> Self {
        Particle {
            pos: pos,
            vel: vel,
        }
    }
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem { particles: Vec::new() }
    }

    pub fn emit(&mut self) {
        let pos = Point2::new(0.0, 0.0);
        let vec = Vector2::new(1.0, 1.0);
        let newparticle = Particle::new(pos, vec);
        self.particles.push(newparticle);
    }

    pub fn update(&mut self, dt: f64) {
        for mut p in self.particles.iter_mut() {
            p.pos += p.vel * dt;
        }
    }
}

impl graphics::Drawable for ParticleSystem {
    fn draw_ex(&self,
               context: &mut Context,
               src: Option<graphics::Rect>,
               dst: Option<graphics::Rect>,
               angle: f64,
               center: Option<graphics::Point>,
               flip_horizontal: bool,
               flip_vertical: bool)
               -> GameResult<()> {
        Ok(())
    }
}
