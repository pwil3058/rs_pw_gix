// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cmp::{Eq, Ordering, PartialEq, PartialOrd};
use std::convert::From;
use std::rc::Rc;

pub use colour_math::{ColourInterface, I_BLUE, I_GREEN, I_RED};
pub type Colour = colour_math::Colour<f64>;
pub type Hue = colour_math::rgb::Hue<f64>;
pub type RGB = colour_math::rgb::RGB<f64>;

pub mod attributes;

