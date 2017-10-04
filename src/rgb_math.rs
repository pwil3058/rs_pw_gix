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
use std::convert::From;
use std::f64::consts;
use std::ops::{Index, Div, Mul, Add, Sub};

use gdk;

use num::{Integer, Float, Num};

macro_rules! is_proportion {
    ( $x:expr ) => {
        {
            ($x <= 1.0) && ($x >= 0.0)
        }
    }
}

macro_rules! is_normalised {
    ( $x:expr ) => {
        {
            ($x <= consts::PI) && ($x >= -consts::PI)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct GRGB<T: Num + PartialOrd> {
    red: T,
    green: T,
    blue: T
}

impl<T: Num + PartialOrd> GRGB<T> {
    pub fn new(red: T, green: T, blue: T) -> GRGB<T> {
        GRGB::<T>{red: red, blue: blue, green: green}
    }

    pub fn indices_value_order(&self) -> (u8, u8, u8) {
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

impl<T: Num + PartialOrd> Add for GRGB<T> {
    type Output = GRGB<T>;

    fn add(self, other: GRGB<T>) -> GRGB<T> {
        GRGB::<T>{
            red: self.red + other.red,
            green: self.green + other.green,
            blue: self.blue + other.blue
        }
    }
}

impl<T: Num + PartialOrd> Sub for GRGB<T> {
    type Output = GRGB<T>;

    fn sub(self, other: GRGB<T>) -> GRGB<T> {
        GRGB::<T>{
            red: self.red - other.red,
            green: self.green - other.green,
            blue: self.blue - other.blue
        }
    }
}

impl<T: Num + PartialOrd, Scalar: Into<T> + Copy> Mul<Scalar> for GRGB<T> {
    type Output = GRGB<T>;

    fn mul(self, rhs: Scalar) -> GRGB<T> {
        GRGB::<T>{
            red: self.red * rhs.into(),
            green: self.green * rhs.into(),
            blue: self.blue * rhs.into()
        }
    }
}

impl<T: Num + PartialOrd, Scalar: Into<T> + Copy> Div<Scalar> for GRGB<T> {
    type Output = GRGB<T>;

    fn div(self, rhs: Scalar) -> GRGB<T> {
        GRGB::<T>{
            red: self.red / rhs.into(),
            green: self.green / rhs.into(),
            blue: self.blue / rhs.into()
        }
    }
}

impl<T: Num + PartialOrd, U: Into<u8>> Index<U> for GRGB<T> {
    type Output = T;

    fn index(&self, u: U) -> &T {
        let index: u8 = u.into();
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

const SIN_120: f64 = 0.86602_54037_844387;
const COS_120: f64 = -0.5;
const COT_120: f64 = 0.5773502691896256;
const RGB_TO_X_VECTOR: [f64; 3] = [1.0, COS_120, COS_120];
const RGB_TO_Y_VECTOR: [f64; 3] = [0.0, SIN_120, -SIN_120];

const PI_30: f64 = consts::FRAC_PI_6;
const PI_45: f64 = consts::FRAC_PI_4;
const PI_60: f64 = consts::FRAC_PI_3;
const PI_120: f64 = consts::FRAC_PI_3 * 2.0;
const PI_180: f64 = consts::PI;

macro_rules! rgb_x_coord {
    ( $rgb:expr ) => {
        {
            $rgb.red + ($rgb.green + $rgb.blue) * COS_120
        }
    }
}

macro_rules! rgb_y_coord {
    ( $rgb:expr ) => {
        {
            ($rgb.green - $rgb.blue) * SIN_120
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct XY {
    x: f64,
    y: f64
}

impl From<XY> for RGB {
    fn from(xy: XY) -> RGB {
        let a = xy.x / COS_120;
        let b = xy.y / SIN_120;
        if xy.y > 0.0 {
            if a > b {
                RGB{red: 0.0, green: (a + b) / 2.0, blue: (a - b) / 2.0}
            } else {
                RGB{red: xy.x - b * COS_120, green: b, blue: 0.0}
            }
        } else if xy.y < 0.0 {
            if a > -b {
                RGB{red: 0.0, green: (a + b) / 2.0, blue: (a - b) / 2.0}
            } else {
                RGB{red: xy.x + b * COS_120, green: 0.0, blue: -b}
            }
        } else if xy.x < 0.0 {
            let ha = a / 2.0;
            RGB{red: 0.0, green: ha, blue: ha}
        } else {
            RGB{red: xy.x, green: 0.0, blue: 0.0}
        }
    }
}

impl From<XY> for RGB16 {
    fn from(xy: XY) -> RGB16 {
        RGB16::from(RGB::from(xy))
    }
}

impl From<XY> for RGB8 {
    fn from(xy: XY) -> RGB8 {
        RGB8::from(RGB::from(xy))
    }
}

impl From<RGB> for XY {
    fn from(rgb: RGB) -> XY {
        XY{x: rgb_x_coord!(rgb), y: rgb_y_coord!(rgb)}
    }
}

impl RGB {
    pub fn all_are_proportions(&self) -> bool {
        is_proportion!(self.red) && is_proportion!(self.green) && is_proportion!(self.blue)
    }

    fn sum(&self) -> f64 {
        (self.red + self.green + self.blue)
    }

    pub fn value(&self) -> f64 {
        self.sum() / 3.0
    }

    pub fn agree_within_limit(&self, other: &RGB, limit: f64) -> bool {
        true
    }

    fn ff(&self, indices: (u8, u8), ks: (f64, f64)) -> f64 {
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
    pub fn components_rotated(&self, delta_hue_angle: f64) -> RGB {
        assert!(is_normalised!(delta_hue_angle));
        fn calc_ks(delta_hue_angle: f64) -> (f64, f64) {
            let a = delta_hue_angle.sin();
            let b = (PI_120 - delta_hue_angle).sin();
            let c = a + b;
            (b / c, a / c)
        }
        if delta_hue_angle > 0.0 {
            if delta_hue_angle > PI_120 {
                let ks = calc_ks(delta_hue_angle - PI_120);
                return RGB{red: self.ff((2, 1), ks), green: self.ff((0, 2), ks), blue: self.ff((1, 0), ks)}
            } else {
                let ks = calc_ks(delta_hue_angle);
                return RGB{red: self.ff((0, 2), ks), green: self.ff((1, 0), ks), blue: self.ff((2, 1), ks)}
            }
        } else if delta_hue_angle < 0.0 {
            if delta_hue_angle < -PI_120 {
                let ks = calc_ks(delta_hue_angle.abs() - PI_120);
                return RGB{red: self.ff((1, 2), ks), green: self.ff((2, 0), ks), blue: self.ff((0, 1), ks)}
            } else {
                let ks = calc_ks(delta_hue_angle.abs());
                return RGB{red: self.ff((0, 1), ks), green: self.ff((1, 2), ks), blue: self.ff((2, 0), ks)}
            }
        }
        *self
    }

    // An alternative implementation of components_rotated()
    // TODO: test which implemention of components_rotated() is most
    // effecient
    pub fn components_rotated_alt(&self, delta_hue_angle: f64) -> RGB {
        fn x_rotated(x: f64, theta: f64) -> XY {
            XY{x: x * theta.cos(), y: x * theta.sin()}
        }
        let red_rtd_xy = x_rotated(self.red, delta_hue_angle);
        let green_rtd_xy = x_rotated(self.green, delta_hue_angle + PI_120);
        let blue_rtd_xy = x_rotated(self.blue, delta_hue_angle - PI_120);
        let rgb = RGB::from(red_rtd_xy) + RGB::from(green_rtd_xy) + RGB::from(blue_rtd_xy);
        let new_sum = rgb.sum();
        if new_sum != 0.0 {
            rgb * self.sum() / new_sum
        } else {
            rgb
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn within_limits(rgb1: RGB, rgb2: RGB) -> bool {
        let limit = 0.000000001;
        for i in 0..3 {
            if rgb1[i] != 0.0 {
                if ((rgb2[i] / rgb1[i]) - 1.0).abs() > limit {
                    return false
                }
            } else if rgb2[i] != 0.0 {
                if ((rgb1[i] / rgb2[i]) - 1.0).abs() > limit {
                    return false
                }
            }
        }
        true
    }

    #[test]
    fn rgb_math_grgb_basics() {
        let rgb8 = GRGB::<u8>::new(1, 2, 3);
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
    fn rgb_math_casting() {
        for rgb in [BLACK, WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA].iter() {
            assert!(within_limits(RGB::from(RGB8::from(*rgb)), *rgb));
            assert!(within_limits(RGB::from(RGB16::from(*rgb)), *rgb));
        }
    }

    #[test]
    fn rgb_math_xy_components()  {
        for x in [BLACK, WHITE].iter() {
            assert_eq!(XY::from(*x), XY{x: 0.0, y: 0.0});
        }

        assert_eq!(XY::from(RED), XY{x: 1.0, y: 0.0});
        assert_eq!(XY::from(GREEN), XY{x: COS_120, y: SIN_120});
        assert_eq!(XY::from(BLUE), XY{x: COS_120, y: -SIN_120});

        assert_eq!(XY::from(YELLOW), XY{x: 1.0 + COS_120, y: SIN_120});
        assert_eq!(XY::from(CYAN), XY{x: 2.0 * COS_120, y: 0.0});
        assert_eq!(XY::from(MAGENTA), XY{x: 1.0 + COS_120, y: -SIN_120});

        assert_eq!(RGB::from(XY::from(WHITE)), BLACK);
        for x in [BLACK, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA].iter() {
            assert_eq!(RGB::from(XY::from(*x)), *x);
            assert_eq!(RGB::from(XY::from(*x / 2.0)), *x / 2.0);
        }
        for ab in vec![(0.1, 1.0), (0.1, 0.9), (0.3, 0.7), (1.0, 0.1), (0.9, 0.1), (0.7, 0.3)].iter() {
            let rgb = RGB::new(ab.0, ab.1, 0.0);
            assert!(within_limits(RGB::from(XY::from(rgb)), rgb));
            let rgb = RGB::new(0.0, ab.1,ab.0);
            assert!(within_limits(RGB::from(XY::from(rgb)), rgb));
            let rgb = RGB::new(ab.0, 0.0, ab.1);
            assert!(within_limits(RGB::from(XY::from(rgb)), rgb));
        }
    }

    #[test]
    fn rgb_math_rotation() {
        // NB using conversion where necessary to account for the fact
        // that floating point is only an approximation of real numbers
        println!{"YELLOW = {:?}: {:?}", YELLOW, RGB16::from(YELLOW)};
        assert_eq!(RGB16{red: std::u16::MAX, green: std::u16::MAX, blue:0}, RGB16::from(YELLOW));
        assert_eq!(RGB16::from((YELLOW).components_rotated(-PI_60)), RGB16::from((RED + WHITE) / 2));
        assert_eq!(RGB16::from(RED.components_rotated(PI_60)), RGB16::from(YELLOW / 2));
        assert_eq!(RGB16::from(RED.components_rotated(PI_120)), RGB16::from(GREEN));
        //assert_eq!(RGB16::from(RED.components_rotated(PI_180)), RGB16::from(CYAN / 2));
        assert!(within_limits(RED.components_rotated(PI_180), CYAN / 2));
        assert_eq!(RGB16::from(RED.components_rotated(-PI_60)), RGB16::from(MAGENTA / 2));
        assert_eq!(RGB16::from(RED.components_rotated(-PI_120)), RGB16::from(BLUE));
        //assert_eq!(RGB16::from(RED.components_rotated(-PI_180)), RGB16::from(CYAN / 2));
        assert!(within_limits(RED.components_rotated(-PI_180), CYAN / 2));

        println!("XXXXXXXXXXXX");
        assert_eq!(RGB16::from(YELLOW.components_rotated(PI_60)), RGB16::from((GREEN + WHITE) * 0.5));
        assert_eq!(RGB16::from(YELLOW.components_rotated(PI_120)), RGB16::from(CYAN));
        //assert_eq!(RGB16::from(YELLOW.components_rotated(PI_180)), RGB16::from((BLUE + WHITE) * 0.5));
        assert!(within_limits(YELLOW.components_rotated(PI_180), (BLUE + WHITE) * 0.5));
        assert_eq!(RGB16::from(YELLOW.components_rotated(-PI_60)), RGB16::from((RED + WHITE) / 2));

        //assert_eq!(RGB16::from(GREEN.components_rotated(PI_60)), RGB16::from(CYAN / 2));
        assert!(within_limits(GREEN.components_rotated(PI_60), CYAN / 2));
        assert_eq!(RGB16::from(GREEN.components_rotated(PI_120)), RGB16::from(BLUE));
        //assert_eq!(RGB16::from(GREEN.components_rotated(PI_180)), RGB16::from(MAGENTA / 2));
        assert!(within_limits(GREEN.components_rotated(PI_180), MAGENTA / 2));
        assert_eq!(RGB16::from(GREEN.components_rotated(-PI_60)), RGB16::from(YELLOW / 2));
        assert_eq!(RGB16::from(GREEN.components_rotated(-PI_120)), RGB16::from(RED));
        //assert_eq!(RGB16::from(GREEN.components_rotated(-PI_180)), RGB16::from(MAGENTA / 2));
        assert!(within_limits(GREEN.components_rotated(-PI_180), MAGENTA / 2));
    }
}
