// Copyright 2017 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std;
use std::cmp::PartialOrd;
use std::convert::From;
use std::ops::{Index, Div, Mul, Add, Sub};

use gdk;

use num::Num;

use ::rgb_math::angle::*;

#[macro_export]
macro_rules! is_proportion {
    ( $x:expr ) => {
        {
            ($x <= 1.0) && ($x >= 0.0)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default)]
pub struct GRGB<T: Num + PartialOrd + Copy> {
    pub red: T,
    pub green: T,
    pub blue: T
}

impl<T: Num + PartialOrd + Copy> From<(T, T, T)> for GRGB<T> {
    fn from(rgb: (T, T, T)) -> GRGB<T> {
        GRGB::<T>{red: rgb.0, green: rgb.1, blue: rgb.2}
    }
}

impl<T: Num + PartialOrd + Copy> From<[T; 3]> for GRGB<T> {
    fn from(rgb: [T; 3]) -> GRGB<T> {
        GRGB::<T>{red: rgb[0], green: rgb[1], blue: rgb[2]}
    }
}

impl<T: Num + PartialOrd + Copy> GRGB<T> {
    //pub fn new(red: T, green: T, blue: T) -> GRGB<T> {
        //GRGB::<T>{red: red, blue: blue, green: green}
    //}

    pub fn indices_value_order(&self) -> (usize, usize, usize) {
        if self.red > self.green {
            if self.red > self.blue {
                if self.green > self.blue {
                    (0, 1, 2)
                }else {
                    (0, 2, 1)
                }
            } else {
                (2, 0, 1)
            }
        } else if self.green > self.blue {
            if self.red > self.blue{
                (1, 0, 2)
            } else {
                (1, 2, 0)
            }
         } else {
            (2, 1, 0)
        }
    }
}

impl<T: Num + PartialOrd + Copy> Add for GRGB<T> {
    type Output = GRGB<T>;

    fn add(self, other: GRGB<T>) -> GRGB<T> {
        GRGB::<T>{
            red: self.red + other.red,
            green: self.green + other.green,
            blue: self.blue + other.blue
        }
    }
}

impl<T: Num + PartialOrd + Copy> Sub for GRGB<T> {
    type Output = GRGB<T>;

    fn sub(self, other: GRGB<T>) -> GRGB<T> {
        GRGB::<T>{
            red: self.red - other.red,
            green: self.green - other.green,
            blue: self.blue - other.blue
        }
    }
}

impl<T: Num + PartialOrd + Copy, Scalar: Into<T> + Copy> Mul<Scalar> for GRGB<T> {
    type Output = GRGB<T>;

    fn mul(self, rhs: Scalar) -> GRGB<T> {
        GRGB::<T>{
            red: self.red * rhs.into(),
            green: self.green * rhs.into(),
            blue: self.blue * rhs.into()
        }
    }
}

impl<T: Num + PartialOrd + Copy, Scalar: Into<T> + Copy> Div<Scalar> for GRGB<T> {
    type Output = GRGB<T>;

    fn div(self, rhs: Scalar) -> GRGB<T> {
        GRGB::<T>{
            red: self.red / rhs.into(),
            green: self.green / rhs.into(),
            blue: self.blue / rhs.into()
        }
    }
}

impl<T: Num + PartialOrd + Copy> Index<usize> for GRGB<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        match index {
            0 => &self.red,
            1 => &self.green,
            2 => &self.blue,
            _ => panic!("{:?}: GRGB index out of range", index)
        }
    }
}

pub type RGB = GRGB<f64>;
pub type RGB8 = GRGB<u8>;
pub type RGB16 = GRGB<u16>;

impl From<gdk::RGBA> for RGB {
    fn from(rgba: gdk::RGBA) -> RGB {
        RGB {
            red: rgba.red,
            green: rgba.green,
            blue: rgba.blue,
        }
    }
}

// NB: attempts to do this generically failed due to conflict with "core" crate
// TODO: try to do type conversions generically (again)
impl From<RGB8> for RGB {
    fn from(rgb8: RGB8) -> RGB {
        let divisor = std::u8::MAX as f64;
        RGB {
            red: (rgb8.red as f64) / divisor,
            green: (rgb8.green as f64) / divisor,
            blue: (rgb8.blue as f64)/ divisor,
        }
    }
}

impl From<RGB16> for RGB {
    fn from(rgb16: RGB16) -> RGB {
        let divisor = std::u16::MAX as f64;
        RGB {
            red: (rgb16.red as f64) / divisor,
            green: (rgb16.green as f64) / divisor,
            blue: (rgb16.blue as f64)/ divisor,
        }
    }
}

impl From<RGB> for RGB8 {
    fn from(rgb: RGB) -> RGB8 {
        let scaled_rgb = rgb * std::u8::MAX;
        RGB8 {
            red: scaled_rgb.red.round() as u8,
            green: scaled_rgb.green.round() as u8,
            blue: scaled_rgb.blue.round() as u8,
        }
    }
}

impl From<RGB> for RGB16 {
    fn from(rgb: RGB) -> RGB16 {
        let scaled_rgb = rgb * std::u16::MAX;
        RGB16 {
            red: scaled_rgb.red.round() as u16,
            green: scaled_rgb.green.round() as u16,
            blue: scaled_rgb.blue.round() as u16,
        }
    }
}

pub const BLACK: RGB = RGB{red: 0.0, green: 0.0, blue: 0.0};
pub const WHITE: RGB = RGB{red: 1.0, green: 1.0, blue: 1.0};

pub const RED: RGB = RGB{red: 1.0, green: 0.0, blue: 0.0};
pub const GREEN: RGB = RGB{red: 0.0, green: 1.0, blue: 0.0};
pub const BLUE: RGB = RGB{red: 0.0, green: 0.0, blue: 1.0};

pub const CYAN: RGB = RGB{red: 0.0, green: 1.0, blue: 1.0};
pub const MAGENTA: RGB = RGB{red: 1.0, green: 0.0, blue: 1.0};
pub const YELLOW: RGB = RGB{red: 1.0, green: 1.0, blue: 0.0};

pub const GREYS: [RGB; 2] = [BLACK, WHITE];
pub const PRIMARIES: [RGB; 3] = [RED, GREEN, BLUE];
pub const SECONDARIES: [RGB; 3] = [CYAN, MAGENTA, YELLOW];

impl RGB {
    pub fn all_are_proportions(&self) -> bool {
        is_proportion!(self.red) && is_proportion!(self.green) && is_proportion!(self.blue)
    }

    pub fn sum(&self) -> f64 {
        (self.red + self.green + self.blue)
    }

    pub fn value(&self) -> f64 {
        self.sum() / 3.0
    }

    pub fn best_foreground_rgb(&self) -> RGB {
        if self.red * 0.299 + self.green * 0.587 + self.blue * 0.114 > 0.5 {
            BLACK
        } else {
            WHITE
        }
    }

    fn ff(&self, indices: (usize, usize), ks: (f64, f64)) -> f64 {
        self[indices.0] * ks.0 + self[indices.1] * ks.1
    }

    //Return a copy of the rgb with each component rotated by the specified
    //angle. This results in an rgb the same value but the hue angle rotated
    //by the specified amount.
    //NB the chroma will change when there are less than 3 non zero
    //components and in the case of 2 non zero components this change may
    //be undesirable.  If it is undesirable it can be avoided by using a
    //higher level wrapper function to adjust/restore the chroma value.
    //In some cases maintaining bof chroma and value will not be
    //possible due to the complex relationship between value and chroma.
    pub fn components_rotated(&self, delta_hue_angle: Angle) -> RGB {
        fn calc_ks(delta_hue_angle: Angle) -> (f64, f64) {
            let a = delta_hue_angle.sin();
            let b = (DEG_120 - delta_hue_angle).sin();
            let c = a + b;
            (b / c, a / c)
        }
        if delta_hue_angle > DEG_0 {
            if delta_hue_angle > DEG_120 {
                let ks = calc_ks(delta_hue_angle - DEG_120);
                return RGB{red: self.ff((2, 1), ks), green: self.ff((0, 2), ks), blue: self.ff((1, 0), ks)}
            } else {
                let ks = calc_ks(delta_hue_angle);
                return RGB{red: self.ff((0, 2), ks), green: self.ff((1, 0), ks), blue: self.ff((2, 1), ks)}
            }
        } else if delta_hue_angle < DEG_0 {
            if delta_hue_angle < -DEG_120 {
                let ks = calc_ks(delta_hue_angle.abs() - DEG_120);
                return RGB{red: self.ff((1, 2), ks), green: self.ff((2, 0), ks), blue: self.ff((0, 1), ks)}
            } else {
                let ks = calc_ks(delta_hue_angle.abs());
                return RGB{red: self.ff((0, 1), ks), green: self.ff((1, 2), ks), blue: self.ff((2, 0), ks)}
            }
        }
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn within_limit_quiet(x1: f64, x2:f64) -> bool {
        let limit = 0.0000000001;
        if x1 == 0.0 || x2 == 0.0 {
            (x2 + x1).abs() < limit
        } else {
            ((x1 / x2) - 1.0).abs() < limit
        }
    }

    fn within_limits(rgb1: RGB, rgb2: RGB) -> bool {
        for i in 0..3 {
            if !within_limit_quiet(rgb1[i], rgb2[i]) {
                println!("{:?} != {:?}", rgb1, rgb2);
                return false
            }
        }
        true
    }

    #[test]
    fn rgb_math_grgb_basics() {
        let rgb8 = GRGB::<u8>::from((1, 2, 3));
        assert_eq!(rgb8[0], rgb8.red);
        assert_eq!(rgb8[1], rgb8.green);
        assert_eq!(rgb8[2], rgb8.blue);
    }

    #[test]
    fn rgb_math_rgb_constants() {
        // Check that 1.0 and 0.0s are in the right places
        for x in [BLACK, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA].iter() {
            assert_eq!(BLACK + *x, *x);
            assert_eq!(*x - BLACK, *x);
            assert!(x.all_are_proportions())
        }
        assert_eq!(RED + GREEN + BLUE, WHITE);
        assert_eq!(RED + GREEN, YELLOW);
        assert_eq!(RED + BLUE, MAGENTA);
        assert_eq!(GREEN + BLUE, CYAN);

        assert_eq!(WHITE - RED, CYAN);
        assert_eq!(WHITE - GREEN, MAGENTA);
        assert_eq!(WHITE - BLUE, YELLOW);

        assert_eq!(BLACK.value(), 0.0);
        assert_eq!(WHITE.value(), 1.0);

         for x in [RED, GREEN, BLUE].iter() {
            assert_eq!(x.value(), 1.0 / 3.0);
        }

        for x in [YELLOW, CYAN, MAGENTA].iter() {
            assert_eq!(x.value(), 2.0 / 3.0);
        }
    }

    #[test]
    fn rgb_math_rgb_casting() {
        for rgb in [BLACK, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA].iter() {
            assert!(within_limits(RGB::from(RGB8::from(*rgb)), *rgb));
            assert!(within_limits(RGB::from(RGB16::from(*rgb)), *rgb));
        }
    }

    #[test]
    fn rgb_math_rgb_rotation() {
        // NB using conversion where necessary to account for the fact
        // that floating point is only an approximation of real numbers
        assert_eq!(RGB16{red: std::u16::MAX, green: std::u16::MAX, blue:0}, RGB16::from(YELLOW));
        assert_eq!(RGB16::from((YELLOW).components_rotated(-DEG_60)), RGB16::from((RED + WHITE) / 2));
        assert_eq!(RGB16::from(RED.components_rotated(DEG_60)), RGB16::from(YELLOW / 2));
        assert_eq!(RGB16::from(RED.components_rotated(DEG_120)), RGB16::from(GREEN));
        //assert_eq!(RGB16::from(RED.components_rotated(DEG_180)), RGB16::from(CYAN / 2));
        assert!(within_limits(RED.components_rotated(DEG_180), CYAN / 2));
        assert_eq!(RGB16::from(RED.components_rotated(-DEG_60)), RGB16::from(MAGENTA / 2));
        assert_eq!(RGB16::from(RED.components_rotated(-DEG_120)), RGB16::from(BLUE));
        //assert_eq!(RGB16::from(RED.components_rotated(-DEG_180)), RGB16::from(CYAN / 2));
        assert!(within_limits(RED.components_rotated(-DEG_180), CYAN / 2));

        assert_eq!(RGB16::from(YELLOW.components_rotated(DEG_60)), RGB16::from((GREEN + WHITE) * 0.5));
        assert_eq!(RGB16::from(YELLOW.components_rotated(DEG_120)), RGB16::from(CYAN));
        //assert_eq!(RGB16::from(YELLOW.components_rotated(DEG_180)), RGB16::from((BLUE + WHITE) * 0.5));
        assert!(within_limits(YELLOW.components_rotated(DEG_180), (BLUE + WHITE) * 0.5));
        assert_eq!(RGB16::from(YELLOW.components_rotated(-DEG_60)), RGB16::from((RED + WHITE) / 2));

        //assert_eq!(RGB16::from(GREEN.components_rotated(DEG_60)), RGB16::from(CYAN / 2));
        assert!(within_limits(GREEN.components_rotated(DEG_60), CYAN / 2));
        assert_eq!(RGB16::from(GREEN.components_rotated(DEG_120)), RGB16::from(BLUE));
        //assert_eq!(RGB16::from(GREEN.components_rotated(DEG_180)), RGB16::from(MAGENTA / 2));
        assert!(within_limits(GREEN.components_rotated(DEG_180), MAGENTA / 2));
        assert_eq!(RGB16::from(GREEN.components_rotated(-DEG_60)), RGB16::from(YELLOW / 2));
        assert_eq!(RGB16::from(GREEN.components_rotated(-DEG_120)), RGB16::from(RED));
        //assert_eq!(RGB16::from(GREEN.components_rotated(-DEG_180)), RGB16::from(MAGENTA / 2));
        assert!(within_limits(GREEN.components_rotated(-DEG_180), MAGENTA / 2));
    }
}
