// Useful references:
// https://www.reddit.com/r/gamedev/comments/13ksu3/article_on_particle_systems_and_an_online_cocos2d/
// Unity3D's particle system
// Cocos2d's plist file format
// Oh, love2d's particle system parameters, derp.

use std::marker::Sized;

use std::f64;


use rand;
use rand::Rng;
use na;
use ggez::{GameResult, Context};
use ggez::graphics;

use super::{Point2, Vector2};

enum StartParam<T> {
    Fixed(T),
    UniformRange(T, T), /* todo: stepped range, a list of discrete values of which one gets chosen. */
}


impl StartParam<f64> {
    pub fn get_value(&self) -> f64 {
        match *self {
            StartParam::Fixed(x) => x,
            StartParam::UniformRange(ref low, ref high) => {
                let mut rng = rand::thread_rng();
                rng.gen_range(*low, *high)
            }
        }
    }
}

// Apparently implementing SampleRange for our own type
// isn't something we should do, so we just define this by hand...
impl StartParam<Vector2> {
    fn get_value(&self) -> Vector2 {
        match *self {
            StartParam::Fixed(x) => x,
            StartParam::UniformRange(low, high) => {
                let mut rng = rand::thread_rng();
                let x = rng.gen_range(low.x, high.x);
                let y = rng.gen_range(low.y, high.y);
                Vector2::new(x, y)
            }
        }
    }
}

impl StartParam<Point2> {
    fn get_value(&self) -> Point2 {
        match *self {
            StartParam::Fixed(x) => x,
            StartParam::UniformRange(low, high) => {
                let mut rng = rand::thread_rng();
                let x = rng.gen_range(low.x, high.x);
                let y = rng.gen_range(low.y, high.y);
                Point2::new(x, y)
            }
        }
    }
}

impl StartParam<graphics::Color> {
    fn get_value(&self) -> graphics::Color {
        match *self {
            StartParam::Fixed(x) => x,
            StartParam::UniformRange(low, high) => {
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

/// A trait that defines a way to do some sort of
/// lerp or easing function on a type.
pub trait Interpable
    where Self: Sized
{
    /// Interpolate the value.  t should always be a number
    /// between 0.0 and 1.0, normalized for whatever actual
    /// value is the "end" of the interpolation.
    fn interp(&self, t: f64) -> Self;

    fn interp_between(t: f64, v1: Self, v2: Self) -> Self;

    /// A little shortcut that does the normalization for you.
    fn normalize_interp(&self, t: f64, max_t: f64) -> Self {
        let norm_t = t / max_t;
        self.interp(norm_t)
    }

    fn normalize_interp_between(t: f64, max_t: f64, v1: Self, v2: Self) -> Self {
        let norm_t = t / max_t;
        Self::interp_between(norm_t, v1, v2)
    }
}

impl Interpable for f64 {
    fn interp(&self, t: f64) -> Self {
        *self * t
    }

    fn interp_between(t: f64, v1: Self, v2: Self) -> Self {
        let val1 = v1.interp(1.0 - t);
        let val2 = v2.interp(t);
        val1 + val2
    }
}


// This function is broken; see ggj2017 code for fix.  :/
impl Interpable for graphics::Color {
    fn interp(&self, t: f64) -> Self {
        //*self * t
        let (r, g, b, a): (u8, u8, u8, u8) = (*self).into();
        let (fr, fg, fb, fa) = (r as f64, g as f64, b as f64, a as f64);
        let (rr, rg, rb, ra) = (fr * t, fg * t, fb * t, fa * t);
        (rr as u8, rg as u8, rb as u8, ra as u8).into()
    }

    fn interp_between(t: f64, v1: Self, v2: Self) -> Self {

        let (r1, g1, b1, a1) = v1.into();
        let (fr1, fg1, fb1, fa1) = (r1 as f64, g1 as f64, b1 as f64, a1 as f64);

        let (r2, g2, b2, a2) = v2.into();

        let dr = (r2 - r1) as f64;
        let dg = (g2 - g1) as f64;
        let db = (b2 - b1) as f64;
        let da = (a2 - a1) as f64;

        let (rr, rg, rb, ra) = (fr1 + dr * t, fg1 + dg * t, fb1 + db * t, fa1 + da * t);
        (rr as u8, rg as u8, rb as u8, ra as u8).into()
    }
}

/// A structure that represents a transition between
/// set properties, with multiple potential defined points.
/// So for instance you could use Transition<Color> and define
/// a transition of colors from red to orange to grey to do smoke.
/// You could also use Transition<f64> to just represent a size
/// curve.
/// So really this is a general-purpose easing type thing...
/// It assumes that all time values range from 0 to 1.
pub enum Transition<T: Copy> {
    Fixed(T),
    Range(T, T),
}


impl<T: Interpable + Copy> Transition<T> {
    pub fn fixed(value: T) -> Self {
        Transition::Fixed(value)
    }

    pub fn range(from: T, to: T) -> Self {
        Transition::Range(from, to)
    }

    /// t should be between 0.0 and 1.0
    /// or should it take the current value and a delta-t???
    pub fn get(&self, t: f64) -> T {
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
// rotation
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
    pos: Point2,
    vel: Vector2,
    color: graphics::Color,
    size: f64,
    angle: f64,
    rotation: f64,
    age: f64,
    max_age: f64,
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
    fn new(pos: Point2,
           vel: Vector2,
           color: graphics::Color,
           size: f64,
           angle: f64,
           max_age: f64)
           -> Self {
        Particle {
            pos: pos,
            vel: vel,
            color: color,
            size: size,
            angle: angle,
            rotation: 0.0,
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
            self.system.$name = StartParam::Fixed($name);
            self
        }

        pub fn $rangename(mut self, start: $typ, end: $typ) -> Self {
            self.system.$name = StartParam::UniformRange(start, end);
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
    prop!(start_size, start_size_range, f64);
    prop!(start_rotation, start_rotation_range, f64);
    // These two need some work, 'cause, shapes.
    prop!(start_position, start_position_range, Point2);
    prop!(start_velocity, start_velocity_range, Vector2);
    prop!(start_max_age, start_max_age_range, f64);

    pub fn acceleration(mut self, accel: Vector2) -> Self {
        self.system.acceleration = accel;
        self
    }

    // This also needs some variety in life.
    pub fn emission_rate(mut self, start: f64) -> Self {
        self.system.emission_rate = start;
        self
    }

    pub fn delta_size(mut self, trans: Transition<f64>) -> Self {
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

pub enum EmissionShape {
    // Source point
    Point(Point2),
    // min and max bounds of the line segment.
    Line(Point2, Point2),
    // Center point and radius
    Circle(Point2, f64),
}

impl EmissionShape {
    /// Gets a random point that complies
    /// with the given shape.
    /// TODO: This is an ideal case for unit tests.
    fn get_random(&self) -> Point2 {
        match *self {
            EmissionShape::Point(v) => v,
            EmissionShape::Line(p1, p2) => {

                let min_x = f64::min(p1.x, p2.x);
                let max_x = f64::max(p1.x, p2.x);
                let min_y = f64::min(p1.y, p2.y);
                let max_y = f64::max(p1.y, p2.y);
                let mut rng = rand::thread_rng();
                let x: f64;
                let y: f64;
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
                // let x_min = f64::max(min.x, f64::min(x_bbox_ymin, x_bbox_ymax));
                // let x_max = f64::min(max.x, f64::max(x_bbox_ymin, x_bbox_ymax));


                // let y_bbox_xmin = y_from_x(min.x);
                // let y_bbox_xmax = y_from_x(max.x);
                // let y_min = f64::max(min.y, f64::min(y_bbox_xmin, y_bbox_xmax));
                // let y_max = f64::min(max.y, f64::max(y_bbox_xmin, y_bbox_xmax));

                Point2::new(x, y)

            }
            EmissionShape::Circle(center, radius) => {
                let mut rng = rand::thread_rng();
                let theta = rng.gen_range(0.0, f64::consts::PI * 2.0);
                let r = rng.gen_range(0.0, radius);
                let x = theta.cos() * r;
                let y = theta.sin() * r;
                center + Vector2::new(x, y)
            }
        }
    }
}

enum EmissionVelocity {
    Uniform,
    Direction,
    Cone,
}


pub struct ParticleSystem {
    // Bookkeeping stuff
    particles: Vec<Particle>,
    residual_particle: f64,
    max_particles: usize,
    image: graphics::Image,

    // Parameters:
    // Emission parameters
    emission_rate: f64,
    start_color: StartParam<graphics::Color>,
    start_position: StartParam<Point2>,
    start_shape: EmissionShape,
    start_velocity: StartParam<Vector2>,
    start_angle: StartParam<f64>,
    start_rotation: StartParam<f64>,
    start_size: StartParam<f64>,
    start_max_age: StartParam<f64>,
    // Global state/update parameters
    acceleration: Vector2,

    delta_size: Transition<f64>,
    delta_color: Transition<graphics::Color>,
}



impl ParticleSystem {
    pub fn new(ctx: &mut Context) -> Self {
        ParticleSystem {
            particles: Vec::new(),
            max_particles: 0,
            image: ParticleSystem::make_image(ctx, 5),
            acceleration: Vector2::new(0.0, 0.0),
            start_color: StartParam::Fixed((255, 255, 255).into()),
            start_position: StartParam::Fixed(Point2::new(0.0, 0.0)),
            start_shape: EmissionShape::Point(Point2::new(0.0, 0.0)),
            start_velocity: StartParam::Fixed(Vector2::new(1.0, 1.0)),
            start_angle: StartParam::Fixed(0.0),
            start_rotation: StartParam::Fixed(0.0),
            start_size: StartParam::Fixed(1.0),
            start_max_age: StartParam::Fixed(1.0),
            emission_rate: 1.0,
            residual_particle: 0.0,

            delta_size: Transition::fixed(1.0),
            delta_color: Transition::fixed((255, 255, 255).into()),
        }
    }

    /// Makes a basic square image to represent a particle
    /// if we need one.  Just doing graphics::rectangle() isn't
    /// good enough 'cause it can't do rotations.
    /// ...buuuuuut we can't appear to conjure one up out of
    /// raw data...
    /// ...in fact, we need the Renderer to even *attempt* to do such a thing.
    /// Bah!
    fn make_image(ctx: &mut Context, size: u16) -> graphics::Image {
        graphics::Image::solid(ctx, size, graphics::Color::from((255, 255, 255, 255))).unwrap()
    }

    pub fn count(&self) -> usize {
        return self.particles.len();
    }

    pub fn emit_one(&mut self) {
        // let pos = self.start_position.get_value();
        let pos = self.start_shape.get_random();
        let vec = self.start_velocity.get_value();
        let col = self.start_color.get_value();
        let size = self.start_size.get_value();
        let max_age = self.start_max_age.get_value();
        let angle = self.start_angle.get_value();
        let rotation = self.start_rotation.get_value();
        let mut newparticle = Particle::new(pos, vec, col, size, angle, max_age);
        newparticle.rotation = rotation;
        if self.particles.len() <= self.max_particles {
            self.particles.push(newparticle);
        }
    }

    pub fn update(&mut self, dt: f64) {
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
            p.vel += self.acceleration * dt;
            p.pos += p.vel * dt;
            p.age += dt;
            p.angle += p.rotation;

            p.size = self.delta_size.get(life_fraction);
            p.color = self.delta_color.get(life_fraction);
        }

        self.particles.retain(|p| p.age < p.max_age);
    }
}

impl graphics::Drawable for ParticleSystem {
    fn draw_ex(&self, context: &mut Context, param: graphics::DrawParam) -> GameResult<()> {
        // BUGGO: Width and height here should be the max bounds of the
        // particle system...?
        // It'd be consistent with our drawing API, but would require
        // finding the bounds of all particles on every tick, which is
        // expensive(ish).
        // Maybe we can make it an x and y scale?  Hmm.
        // let dst_rect = dst.unwrap_or(graphics::Rect::new(0, 0, 0, 0));
        // for p in self.particles.iter() {
        //     // let size = p.size.get_value(life_fraction);
        //     let size = p.size;
        //     let rect = graphics::Rect::new(dst_rect.x() + p.pos.x,
        //                                    dst_rect.y() + p.pos.y,
        //                                    size,
        //                                    size as u32);
        //     graphics::draw(&self.image,
        //     self.image
        //         .draw_ex(context,
        //                  None,
        //                  Some(rect),
        //                  p.angle,
        //                  center,
        //                  flip_horizontal,
        //                  flip_vertical)?;
        // }
        Ok(())
    }
}
