// Useful references:
// https://www.reddit.com/r/gamedev/comments/13ksu3/article_on_particle_systems_and_an_online_cocos2d/
// Unity3D's particle system
// Cocos2d's plist file format
// Oh, love2d's particle system parameters, derp.

// I think this could be simplified.
// For each particle property, take an easing function (perhaps just from the `ezing` crate),
// and bounds to map the start and end to.
// Randomization would alter the bounds per-particle.
// Don't try to cover all cases right off the bat, it should be easy to add more.
//
// The real useful stuff here worth preserving is probably the emission stuff
// and the Particle type...
// StartParam might actually be useful as well maybe.

use std::marker::Sized;

use std::f32;

use ggez::graphics;
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::BlendMode;
use ggez::mint::{Point2, Vector2};
use ggez::{Context, GameResult};
use rand;
use rand::Rng;

enum ValueGenerator<T> {
    Fixed(T),

    // TODO: stepped range, a list of discrete values of which one gets chosen.
    UniformRange(T, T),
}

impl ValueGenerator<f32> {
    pub fn get_value(&self) -> f32 {
        match *self {
            ValueGenerator::Fixed(x) => x,
            ValueGenerator::UniformRange(ref low, ref high) => {
                let mut rng = rand::thread_rng();
                rng.gen_range(*low, *high)
            }
        }
    }
}

// Apparently implementing SampleRange for our own type
// isn't something we should do, so we just define this by hand...
impl ValueGenerator<Vector2<f32>> {
    fn get_value(&self) -> Vector2<f32> {
        match *self {
            ValueGenerator::Fixed(x) => x,
            ValueGenerator::UniformRange(low, high) => {
                let mut rng = rand::thread_rng();
                let x = rng.gen_range(low.x, high.x);
                let y = rng.gen_range(low.y, high.y);
                Vector2 { x, y }
            }
        }
    }
}

impl ValueGenerator<Point2<f32>> {
    fn get_value(&self) -> Point2<f32> {
        match *self {
            ValueGenerator::Fixed(x) => x,
            ValueGenerator::UniformRange(low, high) => {
                let mut rng = rand::thread_rng();
                let x = rng.gen_range(low.x, high.x);
                let y = rng.gen_range(low.y, high.y);
                Point2 { x, y }
            }
        }
    }
}

impl ValueGenerator<graphics::Color> {
    fn get_value(&self) -> graphics::Color {
        match *self {
            ValueGenerator::Fixed(x) => x,
            ValueGenerator::UniformRange(low, high) => {
                let mut rng = rand::thread_rng();
                let (lowr, lowg, lowb) = low.into();
                let (hir, hig, hib) = high.into();
                let r = rng.gen_range(lowr, hir);
                let g = rng.gen_range(lowg, hig);
                let b = rng.gen_range(lowb, hib);
                (r, g, b).into()
            }
        }
    }
}

pub type EasingFn = Fn(f32) -> f32;

/// Linear interpolation; assumes input value is in the range 0-1 and
/// returns it interpolated to the given bounds.
///
/// For example: `lerp(easing::cubic_inout(v), 0.0, 100.0)`
pub fn lerp(v: f32, from: f32, to: f32) -> f32 {
    let delta = to - from;
    v * delta
}

/// A trait that defines a way to do some sort of
/// lerp or easing function on a type.
pub trait Interpolate
where
    Self: Sized,
{
    /// Interpolate the value.  t should always be a number
    /// between 0.0 and 1.0, normalized for whatever actual
    /// value is the "end" of the interpolation.
    fn interp(&self, t: f32) -> Self;

    fn interp_between(t: f32, v1: Self, v2: Self) -> Self;

    /// A little shortcut that does the normalization for you.
    fn normalize_interp(&self, t: f32, max_t: f32) -> Self {
        let norm_t = t / max_t;
        self.interp(norm_t)
    }

    /// Combines interp_between with normalize_interp()
    fn normalize_interp_between(t: f32, max_t: f32, v1: Self, v2: Self) -> Self {
        let norm_t = t / max_t;
        Self::interp_between(norm_t, v1, v2)
    }
}

impl Interpolate for f32 {
    fn interp(&self, t: f32) -> Self {
        *self * t
    }

    fn interp_between(t: f32, v1: Self, v2: Self) -> Self {
        let val1 = v1.interp(1.0 - t);
        let val2 = v2.interp(t);
        val1 + val2
    }
}

// This function is broken; see ggj2017 code for fix.  :/
impl Interpolate for graphics::Color {
    fn interp(&self, t: f32) -> Self {
        let rt = self.r * t;
        let gt = self.g * t;
        let bt = self.b * t;
        let at = self.a * t;
        graphics::Color::new(rt, gt, bt, at)
    }

    fn interp_between(t: f32, v1: Self, v2: Self) -> Self {
        let val1 = v1.interp(1.0 - t);
        let val2 = v2.interp(t);
        let r = val1.r + val2.r;
        let g = val1.g + val2.g;
        let b = val1.b + val2.b;
        let a = val1.a + val2.a;
        graphics::Color::new(r, g, b, a)
    }
}

/// A structure that represents a transition between
/// set properties, with multiple potential defined points.
/// So for instance you could use Transition<Color> and define
/// a transition of colors from red to orange to grey to do smoke.
/// You could also use Transition<f32> to just represent a size
/// curve.
/// So really this is a general-purpose easing type thing...
/// It assumes that all time values range from 0 to 1.
pub enum Transition<T: Copy> {
    Fixed(T),
    Range(T, T),
}

impl<T: Interpolate + Copy> Transition<T> {
    pub fn fixed(value: T) -> Self {
        Transition::Fixed(value)
    }

    pub fn range(from: T, to: T) -> Self {
        Transition::Range(from, to)
    }

    /// t should be between 0.0 and 1.0
    /// or should it take the current value and a delta-t???
    pub fn get(&self, t: f32) -> T {
        match *self {
            Transition::Fixed(value) => value,
            Transition::Range(from, to) => T::interp_between(t, from, to),
        }
    }
}

// Properties particles should have:
// Age, position, velocity

// Properties particle systems should have:
// color, inc. opacity
// texture (perhaps sprite?), multiplied by color
// size
// gravity
// fade rate/color transitions
// max lifespan
// speed
// xvel, yvel
// shape???
// Gravity???
// Glow???
// x/y bounds (delete particles that go further than this)
// floor and ceiling?  (particles bounce off of these)
//
// Per love2d, which appears to cover all the basics and more:
// area spread (uniform, normal)
// * buffer size (number of particles)
// * linear acceleration (general case of gravity)
// color (of image)
// colors (of non-image particle)
// direction
// emission rate (constant, burst)
// emitter lifetime
// image
// insert mode (where particles are inserted; top, bottom, random)
// lifetime
// linear damping
// particle lifetime (min, max)
// position of emitter
// quads (series of images to use as sprites)
// radial acceeleration
// ang_vel
// size variations/sizes
// set speed
// spin, spin variation
// spread
// tangential acceleration
//
// Honestly having general purpose "create" and "update" traits
// would abstract out a lot of this, and then we just define
// the basics.
//
// It would also be very nice to be able to have a particle system
// calculate in is own relative coordinate space OR world absolute space.
// Though if the user defines their own worldspace coordinate system
// that could get a bit sticky.  :/

struct Particle {
    pos: Point2<f32>,
    vel: Vector2<f32>,
    color: graphics::Color,
    size: f32,
    angle: f32,
    ang_vel: f32,
    age: f32,
    max_age: f32,
}

// Aha.  We have a 2x2 matrix of cases here: A particle can have a property
// that's specific to each particle and calculated from some particle-specific
// state, like position.  It can have a property that's the same for  each particle
// but calculated the same for each particle, like color in a simple flame effect.
// It can have a property that's not specific to each particle and calculated the
// same for each particle, like gravity, or that's not specific to each particle and
// calculated
//
// So our axes are: State per particle vs state per particle system,
// and constant over time vs varying over time.
//
// The TRICK is that a property can optionally fit into more than one
// of these values, so it has to decide at runtime.  And doing that
// efficiently is a pain in the ass.  >:-[
// So SCREW it, we will handle the most general case.  Bah!
//
// Hmmmm, we could handle this in a more functional way, where we define
// each transition as a function, and then compose/chain them.  But Rust
// requires these functions to be pure.
//
// Okay, any thing that could be a Transition?  We really want that to
// be a per-particle-system thing rather than a per-particle thing.
// Also it's going to be a huge pain in the ass to get the numbers
// right.  :/
//
// While a completely valid insight that's the absolute wrong way of doing it.
// The thing about particle systems that makes them useful is they're fast, and
// the thing that makes them fast is the each particle more or less obeys the
// same rules as all the others.

impl Particle {
    fn new(
        pos: Point2<f32>,
        vel: Vector2<f32>,
        color: graphics::Color,
        size: f32,
        angle: f32,
        max_age: f32,
    ) -> Self {
        Particle {
            pos: pos,
            vel: vel,
            color: color,
            size: size,
            angle: angle,
            ang_vel: 0.0,
            age: 0.0,
            max_age: max_age,
        }
    }
}

// This probably isn't actually needed as a separate type,
// at least at this point,
// but it makes things clearer for the moment...  Hmm.
// Wow the macro system is kind of shitty though, since you
// can't synthesize identifiers.
pub struct ParticleSystemBuilder {
    system: ParticleSystem,
}

macro_rules! prop {
    ($name:ident, $rangename:ident, $typ:ty) => {
        pub fn $name(mut self, $name: $typ) -> Self {
            self.system.$name = ValueGenerator::Fixed($name);
            self
        }

        pub fn $rangename(mut self, start: $typ, end: $typ) -> Self {
            self.system.$name = ValueGenerator::UniformRange(start, end);
            self
        }
    }
}

impl ParticleSystemBuilder {
    pub fn new(ctx: &mut Context) -> Self {
        let system = ParticleSystem::new(ctx);
        ParticleSystemBuilder { system: system }
    }
    pub fn build(self) -> ParticleSystem {
        self.system
    }

    /// Set maximum number of particles.
    pub fn count(mut self, count: usize) -> Self {
        self.system.max_particles = count;
        self.system.particles.reserve_exact(count);
        self
    }

    prop!(start_color, start_color_range, graphics::Color);
    prop!(start_size, start_size_range, f32);
    prop!(start_ang_vel, start_ang_vel_range, f32);
    // These two need some work, 'cause, shapes.
    prop!(start_position, start_position_range, Point2<f32>);
    prop!(start_velocity, start_velocity_range, Vector2<f32>);
    prop!(start_max_age, start_max_age_range, f32);

    pub fn acceleration(mut self, accel: Vector2<f32>) -> Self {
        self.system.acceleration = accel;
        self
    }

    // This also needs some variety in life.
    pub fn emission_rate(mut self, start: f32) -> Self {
        self.system.emission_rate = start;
        self
    }

    pub fn delta_size(mut self, trans: Transition<f32>) -> Self {
        self.system.delta_size = trans;
        self
    }

    pub fn delta_color(mut self, trans: Transition<graphics::Color>) -> Self {
        self.system.delta_color = trans;
        self
    }

    pub fn emission_shape(mut self, shape: EmissionShape) -> Self {
        self.system.start_shape = shape;
        self
    }
}

/// Defines where a new particle should be created.
/// TODO: This basic idea should be used for both initial position
/// and initial velocity...  Uniform, direction, cone, line...
pub enum EmissionShape {
    // Source point
    Point(Point2<f32>),
    // min and max bounds of the line segment.
    Line(Point2<f32>, Point2<f32>),
    // Center point and radius
    Circle(Point2<f32>, f32),
}

impl EmissionShape {
    /// Gets a random point that complies
    /// with the given shape.
    /// TODO: This is an ideal case for unit tests.
    fn get_random(&self) -> Point2<f32> {
        match *self {
            EmissionShape::Point(v) => v,
            EmissionShape::Line(p1, p2) => {
                let min_x = f32::min(p1.x, p2.x);
                let max_x = f32::max(p1.x, p2.x);
                let min_y = f32::min(p1.y, p2.y);
                let max_y = f32::max(p1.y, p2.y);
                let mut rng = rand::thread_rng();
                let x: f32;
                let y: f32;
                if min_x == max_x {
                    // Line is vertical
                    x = min_x;
                    y = rng.gen_range(min_y, max_y);
                } else if min_y == max_y {
                    // Line is horizontal
                    y = max_y;
                    x = rng.gen_range(min_x, max_x)
                } else {
                    // Line is sloped.
                    let dy = max_y - min_y;
                    let dx = max_x - min_x;
                    let slope = dy / dx;
                    x = rng.gen_range(min_x, max_x);
                    y = (slope * (x - min_x)) + min_y;
                }

                // This is a bit sticky 'cause we have
                // to find the min and max x and y that are
                // within the given bounding box
                // let x_bbox_ymin = x_from_y(min.y);
                // let x_bbox_ymax = x_from_y(max.y);
                // let x_min = f32::max(min.x, f32::min(x_bbox_ymin, x_bbox_ymax));
                // let x_max = f32::min(max.x, f32::max(x_bbox_ymin, x_bbox_ymax));

                // let y_bbox_xmin = y_from_x(min.x);
                // let y_bbox_xmax = y_from_x(max.x);
                // let y_min = f32::max(min.y, f32::min(y_bbox_xmin, y_bbox_xmax));
                // let y_max = f32::min(max.y, f32::max(y_bbox_xmin, y_bbox_xmax));

                Point2 { x, y }
            }
            EmissionShape::Circle(center, radius) => {
                let mut rng = rand::thread_rng();
                let theta = rng.gen_range(0.0, f32::consts::PI * 2.0);
                let r = rng.gen_range(0.0, radius);
                let x = theta.cos() * r;
                let y = theta.sin() * r;
                Point2 { x: x + center.x, y: y + center.y }
            }
        }
    }
}

use std::cell::{Cell, RefCell};

pub struct ParticleSystem {
    // Bookkeeping stuff
    particles: Vec<Particle>,
    residual_particle: f32,
    max_particles: usize,

    // Parameters:
    // Emission parameters
    emission_rate: f32,
    start_color: ValueGenerator<graphics::Color>,
    start_position: ValueGenerator<Point2<f32>>,
    start_shape: EmissionShape,
    start_velocity: ValueGenerator<Vector2<f32>>,
    start_angle: ValueGenerator<f32>,
    start_ang_vel: ValueGenerator<f32>,
    start_size: ValueGenerator<f32>,
    start_max_age: ValueGenerator<f32>,
    // Global state/update parameters
    acceleration: Vector2<f32>,

    delta_size: Transition<f32>,
    delta_color: Transition<graphics::Color>,

    sprite_batch: RefCell<SpriteBatch>,
    sprite_batch_dirty: Cell<bool>,
}

impl ParticleSystem {
    pub fn new(ctx: &mut Context) -> Self {
        let image = ParticleSystem::make_image(ctx, 5);
        let sprite_batch = SpriteBatch::new(image);
        ParticleSystem {
            particles: Vec::new(),
            max_particles: 0,
            acceleration: Vector2 { x: 0.0, y: 0.0 },
            start_color: ValueGenerator::Fixed((255, 255, 255).into()),
            start_position: ValueGenerator::Fixed(Point2 { x: 0.0, y: 0.0 }),
            start_shape: EmissionShape::Point(Point2 { x: 0.0, y: 0.0 }),
            start_velocity: ValueGenerator::Fixed(Vector2 { x: 1.0, y: 1.0 }),
            start_angle: ValueGenerator::Fixed(0.0),
            start_ang_vel: ValueGenerator::Fixed(0.0),
            start_size: ValueGenerator::Fixed(1.0),
            start_max_age: ValueGenerator::Fixed(1.0),
            emission_rate: 1.0,
            residual_particle: 0.0,

            delta_size: Transition::fixed(1.0),
            delta_color: Transition::fixed((255, 255, 255).into()),

            sprite_batch: RefCell::new(sprite_batch),
            sprite_batch_dirty: Cell::new(true),
        }
    }

    /// Makes a basic square image to represent a particle
    /// if we need one.
    fn make_image(ctx: &mut Context, size: u16) -> graphics::Image {
        graphics::Image::solid(ctx, size, graphics::Color::from((255, 255, 255, 255))).unwrap()
    }

    /// Number of living particles.
    pub fn count(&self) -> usize {
        return self.particles.len();
    }

    pub fn emit_one(&mut self) {
        let pos = self.start_shape.get_random();
        let vec = self.start_velocity.get_value();
        let col = self.start_color.get_value();
        let size = self.start_size.get_value();
        let max_age = self.start_max_age.get_value();
        let angle = self.start_angle.get_value();
        let ang_vel = self.start_ang_vel.get_value();
        let mut newparticle = Particle::new(pos, vec, col, size, angle, max_age);
        newparticle.ang_vel = ang_vel;
        if self.particles.len() <= self.max_particles {
            self.particles.push(newparticle);
        }
    }

    pub fn update(&mut self, dt: f32) {
        // This is tricky 'cause we have to keep the emission rate
        // correct and constant.  So we "accumulate" particles over
        // time until we have >1 of them and then emit it.
        let num_to_emit = self.emission_rate * dt + self.residual_particle;
        let actual_num_to_emit = num_to_emit.trunc() as usize;
        self.residual_particle = num_to_emit.fract();
        for _ in 0..actual_num_to_emit {
            self.emit_one()
        }
        for mut p in self.particles.iter_mut() {
            let life_fraction = p.age / p.max_age;
            p.vel.x += self.acceleration.x * dt;
            p.vel.y += self.acceleration.y * dt;
            p.pos.x += p.vel.x * dt;
            p.pos.y += p.vel.y * dt;
            p.age += dt;
            p.angle += p.ang_vel;

            p.size = self.delta_size.get(life_fraction);
            p.color = self.delta_color.get(life_fraction);
        }

        self.particles.retain(|p| p.age < p.max_age);
        self.sprite_batch_dirty.set(true);
    }
}

impl graphics::Drawable for ParticleSystem {
    fn draw(&self, context: &mut Context, param: graphics::DrawParam) -> GameResult<()> {
        // Check whether an update has been processed since our last draw call.
        if self.sprite_batch_dirty.get() {
            use std::ops::DerefMut;
            let mut sb_ref = self.sprite_batch.borrow_mut();
            let sb = sb_ref.deref_mut();
            sb.clear();
            for particle in &self.particles {
                let drawparam = graphics::DrawParam {
                    dest: particle.pos,
                    rotation: particle.angle,
                    scale: Vector2 {
                        x: particle.size,
                        y: particle.size,
                    },
                    offset: Point2 { x: 0.5, y: 0.5 },
                    color: particle.color,
                    ..Default::default()
                };
                sb.add(drawparam);
            }
            self.sprite_batch_dirty.set(false);
        }

        self.sprite_batch.borrow().draw(context, param)?;
        Ok(())
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        self.sprite_batch.borrow().blend_mode()
    }

    fn set_blend_mode(&mut self, mode: Option<BlendMode>) {
        self.sprite_batch.borrow_mut().set_blend_mode(mode)
    }

    fn dimensions(&self, _ctx: &mut Context) -> Option<graphics::Rect> {
        if self.particles.is_empty() {
            None
        } else {
            let mut x = f32::MAX;
            let mut y = f32::MAX;
            let mut size = f32::MIN;

            for particle in &self.particles {
                if particle.pos.x < x {
                    x = particle.pos.x;
                }
                if particle.pos.y < y {
                    y = particle.pos.y;
                }
                if particle.size > size {
                    size = particle.size;
                }
            }

            Some(graphics::Rect::new(x, y, size, size))
        }
    }
}
