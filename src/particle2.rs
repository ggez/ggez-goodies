//! Basic particle system.
//!
//! It'd be cool to use Rayon for it someday!

use euclid;
use ggez;
use ggez::graphics;

pub trait Particle {
    fn new() -> Self;
    fn to_draw_param(&self) -> graphics::DrawParam;
    fn update(&mut self, dt: f32);
    fn alive(&self) -> bool;
}

/// A VERY simple particle emitter.
///
/// Need to think about how to make it better.
pub struct Emitter {
    /// Delay between emitting particles.
    /// We use f32 instead of Duration because speed is
    /// more important than precision.
    /// A u32 of nanoseconds or such might be faster, idk.
    delay: f32,

    /// Time since we last emitted a particle.
    last_emitted: f32,
}

impl Emitter {
    pub fn new(rate: f32) -> Self {
        // https://github.com/rust-lang/rust/issues/54361
        // :|
        let delay = 1.0 / rate;
        Self {
            delay,
            last_emitted: 0.0,
        }
    }

    /// This is a sorta weird/lame way of doing it, but it works for now.
    /// Just call this in a loop until it returns `None`.
    fn update<P>(&mut self, dt: f32) -> Option<P>
    where
        P: Particle,
    {
        self.last_emitted -= dt;
        if self.last_emitted < 0.0 {
            self.last_emitted += self.delay;
            Some(P::new())
        } else {
            None
        }
    }
}

pub struct ParticleSystem<P>
where
    P: Particle,
{
    particles: Vec<P>,
    max_particles: usize,
    batch: graphics::spritebatch::SpriteBatch,
    emitter: Emitter,
}

impl<P> ParticleSystem<P>
where
    P: Particle,
{
    pub fn new(limit: usize, emitter: Emitter, image: graphics::Image) -> Self {
        Self {
            particles: Vec::with_capacity(limit),
            max_particles: limit,
            batch: graphics::spritebatch::SpriteBatch::new(image),
            emitter,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Remove old particles
        let mut i = 0;
        while i < self.particles.len() {
            if !self.particles[i].alive() {
                // Remove it and test the particle now
                // in this position.
                self.particles.swap_remove(i);
            } else {
                // Move on to the next particle.
                i += 1;
            }
        }

        // Add new particles, up to the limit
        while self.particles.len() < self.max_particles {
            if let Some(p) = self.emitter.update(dt) {
                self.particles.push(p);
            } else {
                break;
            }
        }

        // Update draw info
        self.batch.clear();
        for p in &mut self.particles {
            p.update(dt);
            self.batch.add(p.to_draw_param());
        }
    }

    /// Returns number of living particles.
    pub fn count(&self) -> usize {
        self.particles.len()
    }
}

impl<P> graphics::Drawable for ParticleSystem<P>
where
    P: Particle,
{
    fn draw(&self, ctx: &mut ggez::Context, param: graphics::DrawParam) -> ggez::GameResult {
        self.batch.draw(ctx, param)
    }

    /// This is kinda odd 'cause tiles don't *strictly* all need to be the same size...
    /// TODO: Find out if Tiled can ever create ones that aren't.
    fn dimensions(&self, ctx: &mut ggez::Context) -> Option<graphics::Rect> {
        self.batch.dimensions(ctx)
    }

    fn set_blend_mode(&mut self, mode: Option<graphics::BlendMode>) {
        self.batch.set_blend_mode(mode);
    }
    fn blend_mode(&self) -> Option<graphics::BlendMode> {
        self.batch.blend_mode()
    }
}

#[derive(Copy, Clone)]
pub struct DefaultParticle {
    pos: crate::Point2,
    vel: crate::Vector2,
    color: graphics::Color,
    size: f32,
    angle: f32,
    ang_vel: f32,
    age: f32,
    max_age: f32,
}

impl Particle for DefaultParticle {
    fn new() -> Self {
        Self {
            pos: euclid::point2(0.0, 0.0),
            vel: euclid::vec2(0.0, 0.0),
            color: graphics::WHITE,
            size: 1.0,
            angle: 0.0,
            ang_vel: 0.0,
            age: 0.0,
            max_age: 10.0,
        }
    }
    fn to_draw_param(&self) -> graphics::DrawParam {
        graphics::DrawParam::default()
            .dest(self.pos)
            .color(self.color)
            .scale(euclid::vec2::<f32, euclid::UnknownUnit>(
                self.size, self.size,
            ))
            .rotation(self.angle)
            .offset(euclid::point2::<f32, euclid::UnknownUnit>(
                self.size / 2.0,
                self.size / 2.0,
            ))
    }
    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.angle += self.ang_vel * dt;
        self.age += dt;
    }
    fn alive(&self) -> bool {
        self.age < self.max_age
    }
}
