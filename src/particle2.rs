use euclid;
use ggez;
use ggez::graphics;
use std::time::{Duration, Instant};

pub trait Particle {
    fn new() -> Self;
    fn to_draw_param(&self) -> graphics::DrawParam;
    fn update(&mut self, dt: Duration);
    fn alive(&self) -> bool;
}

/// A VERY simple particle emitter.
///
/// Need to think about how to make it better.
pub struct Emitter {
    /// Delay between emitting particles.
    delay: Duration,

    last_emitted: Instant,
}

impl Emitter {
    pub fn new(rate: f64) -> Self {
        // https://github.com/rust-lang/rust/issues/54361
        // :|
        let delay_seconds = 1.0 / rate;
        let delay = Duration::from_nanos((delay_seconds * 10e9) as u64);
        Self {
            delay,
            last_emitted: Instant::now(),
        }
    }

    /// This is a sorta weird/lame way of doing it, but it works for now.
    /// Just call this in a loop until it returns `None`.
    ///
    /// TODO: It should probably take a dt instead of using `Instant::now()`
    fn update<P>(&mut self) -> Option<P>
    where
        P: Particle,
    {
        let now = Instant::now();
        let diff = now - self.last_emitted;
        if diff > self.delay {
            self.last_emitted -= self.delay;
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

    pub fn update(&mut self, dt: Duration) {
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
            if let Some(p) = self.emitter.update() {
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
    fn update(&mut self, _dt: Duration) {
        self.pos += self.vel;
        self.angle += self.ang_vel;
        // TODO: Age and such.
        // Do we really want it to be a Duration, or just f32/f64?
        // The fewer conversions we do the better.
        // These are used for visual fx though so we want to use real time,
        // not just tick it once per frame or such.
    }
    fn alive(&self) -> bool {
        self.age < self.max_age
    }
}
