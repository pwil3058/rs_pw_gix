// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub use colour_math::{ColourInterface, HueConstants, RGBConstants, I_BLUE, I_GREEN, I_RED};
pub type Colour = colour_math::Colour<f64>;
pub type Hue = colour_math::rgb::Hue<f64>;
pub type RGB = colour_math::rgb::RGB<f64>;

pub fn rgba_from_rgb(rgb: RGB) -> gdk::RGBA {
    gdk::RGBA {
        red: rgb[I_RED],
        blue: rgb[I_BLUE],
        green: rgb[I_GREEN],
        alpha: 1.0,
    }
}

pub mod attributes;
