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
use std::cmp::{Ordering, PartialOrd};
use std::convert::From;
use std::f64::consts;
use std::ops::{Index, Div, Mul, Add, Sub, Neg};

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

macro_rules! normalise {
    ( $f:expr ) => {
        {
            let mut result = $f;
            if result > consts::PI {
                while result > consts::PI {
                    result -= 2.0 * consts::PI;
                }
            } else if result < -consts::PI {
                while result < -consts::PI {
                    result += 2.0 * consts::PI;
                }
            }
            result
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct GRGB<T: Num + PartialOrd + Copy> {
    red: T,
    green: T,
    blue: T
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

const SIN_120: f64 = 0.86602_54037_844387;
const COS_120: f64 = -0.5;
const COT_120: f64 = 0.5773502691896256;
const RGB_TO_X_VECTOR: [f64; 3] = [1.0, COS_120, COS_120];
const RGB_TO_Y_VECTOR: [f64; 3] = [0.0, SIN_120, -SIN_120];

const PI_0: Angle = Angle(0.0);
const PI_60: Angle = Angle(consts::FRAC_PI_3);
const PI_120: Angle = Angle(consts::FRAC_PI_3 * 2.0);
const PI_180: Angle = Angle(consts::PI);

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

impl XY {
    pub fn calculate_chroma(&self) -> f64 {
        self.x.hypot(self.y) * HueAngle::from(*self).chroma_correction
    }
}

impl From<(f64, f64)> for XY {
    fn from(xy: (f64, f64)) -> XY {
        XY{x: xy.0, y: xy.1}
    }
}

impl From<RGB> for XY {
    fn from(rgb: RGB) -> XY {
        XY{x: rgb_x_coord!(rgb), y: rgb_y_coord!(rgb)}
    }
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

    pub fn calculate_chroma(&self) -> f64 {
        XY::from(*self).calculate_chroma()
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
            let b = (PI_120 - delta_hue_angle).sin();
            let c = a + b;
            (b / c, a / c)
        }
        if delta_hue_angle > Angle(0.0) {
            if delta_hue_angle > PI_120 {
                let ks = calc_ks(delta_hue_angle - PI_120);
                return RGB{red: self.ff((2, 1), ks), green: self.ff((0, 2), ks), blue: self.ff((1, 0), ks)}
            } else {
                let ks = calc_ks(delta_hue_angle);
                return RGB{red: self.ff((0, 2), ks), green: self.ff((1, 0), ks), blue: self.ff((2, 1), ks)}
            }
        } else if delta_hue_angle < Angle(0.0) {
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
    pub fn components_rotated_alt(&self, delta_hue_angle: Angle) -> RGB {
        fn x_rotated(x: f64, theta: Angle) -> XY {
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Angle (f64);

impl From<f64> for Angle {
    fn from(f: f64) -> Angle {
        if f.is_nan() {
            Angle(f)
        } else {
            Angle(normalise!(f))
        }
    }
}

impl From<XY> for Angle {
    fn from(xy: XY) -> Angle {
        if xy.x == 0.0 && xy.y == 0.0 {
            Angle(std::f64::NAN)
        } else {
            Angle(xy.y.atan2(xy.x))
        }
    }
}

// Take into account the circular nature of angle values when
// evaluating order anticlockwise is greater
// NB: cannot implement Ord as: a < b and b < c does NOT imply a < c.
impl PartialOrd for Angle {
    fn partial_cmp(&self, other: &Angle) -> Option<Ordering> {
        if self.is_nan() || other.is_nan() {
            None
        } else {
            let diff = *self - *other;
            if diff.0 < 0.0 {
                Some(Ordering::Less)
            } else if diff.0 > 0.0 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Equal)
            }
        }
    }
}

impl Sub for Angle {
    type Output = Angle;

    fn sub(self, other: Angle) -> Angle {
        Angle::from(self.0 - other.0)
    }
}

impl Add for Angle {
    type Output = Angle;

    fn add(self, other: Angle) -> Angle {
        Angle::from(self.0 + other.0)
    }
}

impl<Scalar: Into<f64> + Copy> Mul<Scalar> for Angle {
    type Output = Angle;

    fn mul(self, rhs: Scalar) -> Angle {
        Angle::from(self.0 * rhs.into())
    }
}

impl<Scalar: Into<f64> + Copy> Div<Scalar> for Angle {
    type Output = Angle;

    fn div(self, rhs: Scalar) -> Angle {
        Angle::from(self.0 / rhs.into())
    }
}

impl Neg for Angle {
    type Output = Angle;

    fn neg(self) -> Angle {
        Angle::from(-self.0)
    }
}

impl Angle {
    pub fn is_nan(self) -> bool {
        self.0.is_nan()
    }

    pub fn abs(self) -> Angle {
        Angle::from(self.0.abs())
    }

    pub fn reciprocal(self) -> Angle {
        Angle::from(self.0 + consts::PI)
    }

    pub fn sin(self) -> f64 {
        self.0.sin()
    }

    pub fn cos(self) -> f64 {
        self.0.cos()
    }

    pub fn tan(self) -> f64 {
        self.0.tan()
    }

    pub fn radians(self) -> f64 {
        self.0
    }

    pub fn degrees(self) -> f64 {
        self.0.to_degrees()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct HueAngle {
    angle: Angle,
    max_chroma_rgb: RGB,
    chroma_correction: f64
}

impl PartialEq for HueAngle {
    fn eq(&self, other: &HueAngle) -> bool {
        self.angle.eq(&other.angle)
    }
}

impl PartialOrd for HueAngle {
    fn partial_cmp(&self, other: &HueAngle) -> Option<Ordering> {
        self.angle.partial_cmp(&other.angle)
    }
}

impl Add<Angle> for HueAngle {
    type Output = HueAngle;

    fn add(self, angle: Angle) -> HueAngle {
        HueAngle::from(self.angle + angle)
    }
}

impl Sub<Angle> for HueAngle {
    type Output = HueAngle;

    fn sub(self, angle: Angle) -> HueAngle {
        HueAngle::from(self.angle - angle)
    }
}

impl Sub<HueAngle> for HueAngle {
    type Output = Angle;

    fn sub(self, other: HueAngle) -> Angle {
        self.angle - other.angle
    }
}

impl From<Angle> for HueAngle {
    fn from(angle: Angle) -> HueAngle {
        if angle.is_nan() {
            HueAngle{angle: angle, max_chroma_rgb: WHITE, chroma_correction: 1.0}
        } else {
            fn calc_other(abs_angle: Angle) -> f64 {
                if [PI_0, PI_120].contains(&abs_angle) {
                    0.0
                } else if [PI_60, PI_180].contains(&abs_angle) {
                    1.0
                } else {
                    fn f(angle: Angle) ->f64 {
                        angle.sin() / (PI_120 - angle).sin()
                    };
                    if abs_angle <= PI_60 {
                        f(abs_angle)
                    } else if abs_angle <= PI_120 {
                        f(PI_120 - abs_angle)
                    } else {
                        f(abs_angle - PI_120)
                    }
                }
            }
            let other = calc_other(angle.abs());
            let max_chroma_rgb = if angle >= PI_0 {
                if angle <= PI_60 {
                    RGB::from((1.0, other, 0.0))
                } else if angle <= PI_120 {
                    RGB::from((other, 1.0, 0.0))
                } else {
                    RGB::from((0.0, 1.0, other))
                }
            } else {
                if angle >= -PI_60 {
                    RGB::from((1.0, 0.0, other))
                } else if angle >= -PI_120 {
                    RGB::from((other, 0.0, 1.0))
                } else {
                    RGB::from((0.0, other, 1.0))
                }
            };
            let chroma_correction = (1.0 + other * other -other).sqrt().recip();
            HueAngle{angle, max_chroma_rgb, chroma_correction}
        }
    }
}

impl From<XY> for HueAngle {
    fn from(xy: XY) -> HueAngle {
        HueAngle::from(Angle::from(xy))
    }
}

impl From<RGB> for HueAngle {
    fn from(rgb: RGB) -> HueAngle {
        HueAngle::from(Angle::from(XY::from(rgb)))
    }
}

impl HueAngle {
    // Return the maximum chroma value that can be achieved for an
    // RGB with this hue and the given value
    pub fn max_chroma_for_value(&self, value: f64) -> f64 {
        assert!(is_proportion!(value));
        if self.angle.is_nan() {
            0.0
        } else {
            let mcv = self.max_chroma_rgb.value();
            if mcv >= value { // on the dark side (easy)
                value / mcv
            } else {
                let abs_angle = self.angle.abs();
                if abs_angle < PI_60 {
                    3.0 * (1.0 - value) * self.chroma_correction / (2.0 * abs_angle.cos())
                } else {
                    3.0 * (1.0 - value) * self.chroma_correction / (2.0 * (abs_angle - PI_120).cos())
                }
            }
        }
    }

    pub fn rgb_with_chroma(&self, req_chroma: f64, dark_side: bool) -> RGB {
        assert!(is_proportion!(req_chroma));
        if req_chroma == 0.0 {
            if dark_side { BLACK } else { WHITE }
        } else if dark_side {
            self.max_chroma_rgb * req_chroma
        } else {
            let mut result: [f64; 3] = [1.0, 1.0, 1.0];
            let mut shortfall = (1.0 - req_chroma) * (3.0 - self.max_chroma_rgb.sum());
            let io = self.max_chroma_rgb.indices_value_order();
            // it's simplest to calculate weakest component first
            let other = self.max_chroma_rgb[io.1];
            result[io.2] = shortfall / (2.0 - other);
            result[io.1] = other + shortfall - result[io.2];
            RGB::from(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PI_30: Angle = Angle(consts::FRAC_PI_6);
    const PI_45: Angle = Angle(consts::FRAC_PI_4);
    const PI_90: Angle = Angle(consts::FRAC_PI_2);
    const PI_150: Angle = Angle(consts::FRAC_PI_6 * 5.0);

    fn within_limit_quiet(x1: f64, x2:f64) -> bool {
        let limit = 0.0000000001;
        if x1 == 0.0 || x2 == 0.0 {
            (x2 + x1).abs() < limit
        } else {
            ((x1 / x2) - 1.0).abs() < limit
        }
    }

    fn within_limit(x1: f64, x2:f64) -> bool {
        if within_limit_quiet(x1, x2) {
            true
        } else {
            println!("{:?} != {:?}", x1, x2);
            false
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

    impl RGB {
        fn non_zero_indices(self) -> Vec<usize> {
            let mut v: Vec<usize> = Vec::new();
            for i in 0..3 {
                if self[i] != 0.0 {
                    v.push(i);
                }
            };
            v
        }
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
            let rgb = RGB::from((ab.1,ab.0, 0.0));
            assert!(within_limits(RGB::from(XY::from(rgb)), rgb));
            let rgb = RGB::from((0.0, ab.1,ab.0));
            assert!(within_limits(RGB::from(XY::from(rgb)), rgb));
            let rgb = RGB::from((ab.0, 0.0, ab.1));
            assert!(within_limits(RGB::from(XY::from(rgb)), rgb));
        }
    }

    #[test]
    fn rgb_math_rotation() {
        // NB using conversion where necessary to account for the fact
        // that floating point is only an approximation of real numbers
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

    #[test]
    fn rgb_math_hue_angle_basics() {
        assert_eq!(HueAngle::from(PI_0).max_chroma_rgb, RED);
        assert_eq!(HueAngle::from(PI_60).max_chroma_rgb, YELLOW);
        assert_eq!(HueAngle::from(PI_120).max_chroma_rgb, GREEN);
        assert_eq!(HueAngle::from(PI_180).max_chroma_rgb, CYAN);
        assert_eq!(HueAngle::from(-PI_0).max_chroma_rgb, RED);
        assert_eq!(HueAngle::from(-PI_60).max_chroma_rgb, MAGENTA);
        assert_eq!(HueAngle::from(-PI_120).max_chroma_rgb, BLUE);
        assert_eq!(HueAngle::from(-PI_180).max_chroma_rgb, CYAN);

        assert_eq!(HueAngle::from(PI_60) - PI_60, HueAngle::from(PI_0));
        assert_eq!(HueAngle::from(PI_60) + PI_60, HueAngle::from(PI_120));

        assert!(within_limit((HueAngle::from(PI_120) - HueAngle::from(PI_90)).radians(), PI_30.radians()));

        for angle in [PI_0, PI_60, PI_120, PI_180, -PI_0, -PI_60, -PI_120, -PI_180].iter() {
            assert_eq!(HueAngle::from(*angle).chroma_correction, 1.0);
        };
        for mul in 1..7 {
            let hue_angle = HueAngle::from(PI_30 * mul);
            assert!(within_limit(hue_angle.max_chroma_rgb.calculate_chroma(), 1.0));
        };
        for angle in [PI_30, PI_90].iter() {
            assert_eq!(HueAngle::from(*angle).max_chroma_rgb.non_zero_indices(), vec![0, 1])
        };
        for angle in [-PI_30, -PI_90].iter() {
            assert_eq!(HueAngle::from(*angle).max_chroma_rgb.non_zero_indices(), vec![0, 2])
        };
        for angle in [PI_150, -PI_150].iter() {
            assert_eq!(HueAngle::from(*angle).max_chroma_rgb.non_zero_indices(), vec![1, 2])
        };
        assert_eq!(HueAngle::from(PI_30).max_chroma_rgb.indices_value_order(), (0, 1, 2));
        assert_eq!(HueAngle::from(PI_90).max_chroma_rgb.indices_value_order(), (1, 0, 2));
        assert_eq!(HueAngle::from(PI_150).max_chroma_rgb.indices_value_order(), (1, 2, 0));
        assert_eq!(HueAngle::from(-PI_30).max_chroma_rgb.indices_value_order(), (0, 2, 1));
        assert_eq!(HueAngle::from(-PI_90).max_chroma_rgb.indices_value_order(), (2, 0, 1));
        assert_eq!(HueAngle::from(-PI_150).max_chroma_rgb.indices_value_order(), (2, 1, 0));
        for angle in [PI_30, PI_90, PI_150, -PI_30, -PI_90, -PI_150].iter() {
            let mut hue_angle = HueAngle::from(*angle);
            let mut second_index = hue_angle.max_chroma_rgb.indices_value_order().1;
            assert!(within_limit(hue_angle.max_chroma_rgb[second_index], 0.5));
        }
        for rgb in [WHITE, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA].iter() {
            assert!(within_limits(HueAngle::from(*rgb).max_chroma_rgb,*rgb));
            for m in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
                assert!(within_limits(HueAngle::from(*rgb * *m).max_chroma_rgb, *rgb));
                let tint = (*rgb + WHITE) * 0.5;
                assert!(within_limits(HueAngle::from(tint * *m).max_chroma_rgb, *rgb));
            }
        }
        for g in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
            let mut rgb = RGB::from((1.0, *g, 0.0));
            assert!(within_limits(HueAngle::from(rgb).max_chroma_rgb,rgb));
            for m in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
                assert!(within_limits(HueAngle::from(rgb * *m).max_chroma_rgb, rgb));
                let tint = (rgb + WHITE) * 0.5;
                assert!(within_limits(HueAngle::from(tint * *m).max_chroma_rgb, rgb));
            }
        }
        assert!(within_limits(HueAngle::from(BLACK).max_chroma_rgb,WHITE));
        for rgb in [BLACK, WHITE, WHITE * 0.5].iter() {
            assert!(HueAngle::from(*rgb).angle.is_nan());
        }
    }

    #[test]
    fn rgb_math_hue_angle_max_chroma_for_value() {
        for angle in [PI_0, PI_120, -PI_0, -PI_120].iter() {
            assert_eq!(HueAngle::from(*angle).max_chroma_for_value(1.0), 0.0);
            assert_eq!(HueAngle::from(*angle).max_chroma_for_value(0.0), 0.0);
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(0.5), 0.75));
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(1.0 / 3.0), 1.0));
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(1.0 / 6.0), 0.5));
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(2.0 / 3.0), 0.5));
        };
        for angle in [PI_60, PI_180, -PI_60, -PI_180].iter() {
            assert_eq!(HueAngle::from(*angle).max_chroma_for_value(1.0), 0.0);
            assert_eq!(HueAngle::from(*angle).max_chroma_for_value(0.0), 0.0);
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(0.5), 0.75));
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(2.0 / 3.0), 1.0));
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(1.0 / 3.0), 0.5));
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(5.0 / 6.0), 0.5));
        };
        for angle in [PI_30, PI_90, PI_150, -PI_30, -PI_90, -PI_150].iter() {
            assert_eq!(HueAngle::from(*angle).max_chroma_for_value(1.0), 0.0);
            assert_eq!(HueAngle::from(*angle).max_chroma_for_value(0.0), 0.0);
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(0.5), 1.0));
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(0.25), 0.5));
            assert!(within_limit(HueAngle::from(*angle).max_chroma_for_value(0.75), 0.5));
        };
    }

    #[test]
    fn rgb_math_rgb_with_chroma() {
        for angle in [PI_0, PI_30, PI_60, PI_90, PI_120, PI_150, PI_180, -PI_0, -PI_30 -PI_60, -PI_90, -PI_120, -PI_150, -PI_180].iter() {
            let hue_angle = HueAngle::from(*angle);
            assert_eq!(hue_angle.rgb_with_chroma(0.0, true), BLACK);
            assert_eq!(hue_angle.rgb_with_chroma(0.0, false), WHITE);
            for chroma in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                let rgb_shade = hue_angle.rgb_with_chroma(*chroma, true);
                let rgb_tint = hue_angle.rgb_with_chroma(*chroma, false);
                assert!(rgb_shade.value() <= rgb_tint.value());
                let shade_chroma = rgb_shade.calculate_chroma();
                let tint_chroma = rgb_tint.calculate_chroma();
                assert!(within_limit(shade_chroma, *chroma));
                assert!(within_limit(tint_chroma, *chroma));
                let shade_angle = HueAngle::from(rgb_shade);
                let tint_angle = HueAngle::from(rgb_tint);
                assert!((hue_angle - shade_angle).abs().radians() <= 0.00000000001);
                assert!((hue_angle - tint_angle).abs().radians() <= 0.00000000001);
            }
        }
    }
}
