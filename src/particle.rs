// Useful references:
// https://www.reddit.com/r/gamedev/comments/2vlypg/i_made_a_html5_particle_engine/
// https://www.reddit.com/r/gamedev/comments/135w5u/version_five_of_my_2d_particle_system_is_complete/
// https://www.reddit.com/r/gamedev/comments/13ksu3/article_on_particle_systems_and_an_online_cocos2d/
// Unity3D's particle system
// Cocos2d's plist file format
// Oh, love2d's particle system parameters, derp.

extern crate nalgebra as na;

use ggez::{GameResult, Context};
use ggez::graphics;

type Point2 = na::Point2<f64>;
type Vector2 = na::Vector2<f64>;

struct Particle {
    pos: Point2,
    vel: Vector2,
}

/// A trait that defines a way to do some sort of
/// lerp or easing function on a type.
trait Interpable {
    fn interp(&self, t: f64) -> Self;
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
struct Transition<T:Interpable> {
    breakpoints: Vec<(f64,T)>,
}

impl<T:Interpable> Transition<T> {
    /// Add a new breakpoint to the transition
    /// at time 0 < t < 1
    fn add(&mut self, t: f64, val: T) {
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
// buffer size (number of particles)
// color (of image)
// colors (of non-image particle)
// direction
// emission rate
// emitter lifetime
// image
// insert mode (where particles are inserted; top, bottom, random)
// lifetime
// linear acceleration (general case of gravity)
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
