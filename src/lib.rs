extern crate ggez;
extern crate rand;
extern crate nalgebra as na;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;


pub mod asset;
pub mod camera;
pub mod input;
pub mod particle;
pub mod scene;
pub mod sprite;
pub mod sprite_loader;

type Point2 = na::Point2<f64>;
type Vector2 = na::Vector2<f64>;
type IPoint2 = na::Point2<u32>;
type IVector2 = na::Vector2<u32>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
