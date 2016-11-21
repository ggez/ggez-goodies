// Useful references:
// https://www.reddit.com/r/gamedev/comments/2vlypg/i_made_a_html5_particle_engine/
// https://www.reddit.com/r/gamedev/comments/135w5u/version_five_of_my_2d_particle_system_is_complete/
// https://www.reddit.com/r/gamedev/comments/13ksu3/article_on_particle_systems_and_an_online_cocos2d/
// Unity3D's particle system
// Cocos2d's plist file format
// Oh, love2d's particle system parameters, derp.

use std::marker::Sized;

use std::cmp::PartialOrd;
use std::f64;


use rand;
use rand::{Rand, Rng};
use rand::distributions::range::SampleRange;
extern crate nalgebra as na;

use ggez;
use ggez::{GameResult, Context};
use ggez::graphics;

type Point2 = na::Point2<f64>;
type Vector2 = na::Vector2<f64>;


enum StartParam<T> {
    Fixed(T),
    UniformRange(T, T),
    // todo: stepped range, a list of discrete values of which one gets chosen.
}

impl<T> StartParam<T> 
where T: PartialOrd + SampleRange + Copy {
    pub fn get_value(&self) -> T {
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
                let (lowr, lowg, lowb) = low.rgb();
                let (hir, hig, hib) = high.rgb();
                let r = rng.gen_range(lowr, hir);
                let g = rng.gen_range(lowg, hig);
                let b = rng.gen_range(lowb, hib);
                graphics::Color::RGB(r, g, b)
            }
        }
    }
}

/// A trait that defines a way to do some sort of
/// lerp or easing function on a type.
trait Interpable where Self: Sized {
    /// Interpolate the value.  t should always be a number
    /// between 0.0 and 1.0, normalized for whatever actual
    /// value is the "end" of the interpolation.
    fn interp(&self, t: f64) -> Self;

    /// A little shortcut that does the normalization for you.
    fn normalize_interp(&self, t: f64, max_t: f64) -> Self {
        let norm_t = t / max_t;
        self.interp(norm_t)
    }
}

impl Interpable for f64 {
    fn interp(&self, t: f64) -> Self {
        *self * t
    }
}

/// A structure that represents a transition between
/// set properties, with multiple potential defined points.
/// So for instance you could use Transition<Color> and define
/// a transition of colors from red to orange to grey to do smoke.
/// You could also use Transition<f64> to just represent a size
/// curve.
/// So really this is a general-purpose easing type thing...
/// It assumes that all time values range from 0 to 1, currently.
/// Though we could fix that just by having or finding some kind of
/// scaling factor... hmmmm.  Nah, that should be external to the
/// transition.
struct Transition<T: Interpable> {
    breakpoints: Vec<(f64, T)>,
}

impl<T: Interpable> Transition<T> {
    /// Add a new breakpoint to the transition
    /// at time 0 < t < 1
    fn add(&mut self, t: f64, val: T) {}
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

impl Particle {
    fn new(pos: Point2, vel: Vector2, color: graphics::Color, size: f64, angle: f64, max_age: f64) -> Self {
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
        ParticleSystemBuilder {
            system: system
        }
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
    start_velocity: StartParam<Vector2>,
    start_angle: StartParam<f64>,
    start_rotation: StartParam<f64>,
    start_size: StartParam<f64>,
    start_max_age: StartParam<f64>,
    // Global state/update parameters
    acceleration: Vector2,
}

impl ParticleSystem {
    pub fn new(ctx: &mut Context) -> Self {
        ParticleSystem { 
            particles: Vec::new(), 
            max_particles: 0,
            image: ParticleSystem::make_image(ctx, 5),
            acceleration: Vector2::new(0.0, 0.0),
            start_color: StartParam::Fixed(graphics::Color::RGB(255,255,255)),
            start_position: StartParam::Fixed(Point2::new(0.0, 0.0)),
            start_velocity: StartParam::Fixed(Vector2::new(1.0, 1.0)),
            start_angle: StartParam::Fixed(0.0),
            start_rotation: StartParam::Fixed(0.0),
            start_size: StartParam::Fixed(1.0),
            start_max_age: StartParam::Fixed(1.0),
            emission_rate: 1.0,
            residual_particle: 0.0,
        }
    }

    /// Makes a basic square image to represent a particle
    /// if we need one.  Just doing graphics::rectangle() isn't
    /// good enough 'cause it can't do rotations.
    /// ...buuuuuut we can't appear to conjure one up out of
    /// raw data...
    /// ...in fact, we need the Renderer to even *attempt* to do such a thing.
    /// Bah!
    fn make_image(ctx: &mut Context, size: u32) -> graphics::Image {
        graphics::Image::solid(ctx, size, graphics::Color::RGBA(255,255,255,255)).unwrap()
    }

    pub fn count(&self) -> usize {
        return self.particles.len()
    }

    pub fn emit_one(&mut self) {
        let pos = self.start_position.get_value();
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
            p.vel += self.acceleration * dt;
            p.pos += p.vel * dt;
            p.age += dt;
            p.angle += p.rotation;
        }

        self.particles.retain(|p| p.age < p.max_age);
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
        // BUGGO: Width and height here should be the max bounds of the
        // particle system...?
        // It'd be consistent with our drawing API, but would require
        // finding the bounds of all particles on every tick, which is
        // expensive(ish).
        // Maybe we can make it an x and y scale?  Hmm.
        let dst_rect = dst.unwrap_or(graphics::Rect::new(0, 0, 0, 0));
        for (i,p) in self.particles.iter().enumerate() {
            let rect = graphics::Rect::new(dst_rect.x() + p.pos.x as i32,
                                           dst_rect.y() + p.pos.y as i32,
                                           p.size as u32,
                                           p.size as u32);
            // BUGGO: AIEEEE this requires &mut self which the trait does not allow...
            // Apparently casting an immutable reference to a mutable one is
            // beyond unsafe, and into undefined, so they don't make it easy
            // for you...
            // Interior mutability?
            // Love2D HAD a ColorMode global setting for just this sort
            // of thing, that multiplied/whatever the current color against
            // all drawing (including images, I think), but they got rid
            // of it in 0.9.0 and I'm not sure why.
            unsafe {
                let evil_mutable_self = &mut *(self as *const Self as *mut Self);
                evil_mutable_self.image.set_color_mod(p.color);
            }
            try!(self.image.draw_ex(context, None, Some(rect), p.angle, None, false, false));
            //graphics::set_color(context, p.color);
            //graphics::rectangle(context, graphics::DrawMode::Fill, rect)?;
        }
        Ok(())
    }
}
