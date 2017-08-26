
extern crate ggez;
extern crate rand;
extern crate nalgebra as na;

extern crate serde_json;


pub mod asset;
//pub mod asset2;
pub mod camera;
pub mod imgui;
pub mod input;
pub mod particle;
pub mod scene;
pub mod sprite;

pub type Point2 = na::Point2<f64>;
pub type Vector2 = na::Vector2<f64>;
pub type Matrix3 = na::Matrix3<f64>;
pub type Similarity2 = na::Similarity2<f64>;
pub type Translation2 = na::Translation2<f64>;
pub type Projective2 = na::Projective2<f64>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

