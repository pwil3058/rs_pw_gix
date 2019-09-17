// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

#[macro_use]
pub mod rgb;
#[macro_use]
pub mod rgb_manipulator;
//#[macro_use]
//pub mod angle;
#[macro_use]
pub mod hue;

pub mod angle {
    pub use normalised_angles::AngleConst;
    pub type Angle = normalised_angles::Angle<f64>;
}
