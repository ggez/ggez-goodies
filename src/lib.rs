extern crate ggez;
extern crate rand;
extern crate nalgebra as na;

extern crate serde_json;


pub mod asset;
pub mod asset2;
pub mod camera;
pub mod input;
pub mod particle;
pub mod scene;
pub mod sprite;

pub type Point2 = na::Point2<f64>;
pub type Vector2 = na::Vector2<f64>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
