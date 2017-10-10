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

use ::rgb_math::angle::*;
use ::rgb_math::rgb::*;

macro_rules! is_proportion {
    ( $x:expr ) => {
        {
            ($x <= 1.0) && ($x >= 0.0)
        }
    }
}

const SIN_120: f64 = 0.86602_54037_844387;
const COS_120: f64 = -0.5;

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

pub trait Hypotenuse {
    fn hypot(&self) -> f64;
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct XY {
    pub x: f64,
    pub y: f64
}

impl XY {
    pub fn calculate_chroma(&self) -> f64 {
        self.x.hypot(self.y) * HueAngle::from(*self).chroma_correction
    }
}

impl Hypotenuse for XY {
    fn hypot(&self) -> f64 {
        self.x.hypot(self.y)
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

impl Hypotenuse for RGB {
    fn hypot(&self) -> f64 {
        XY::from(*self).hypot()
    }
}

impl From<XY> for Angle {
    fn from(xy: XY) -> Angle {
        if xy.x == 0.0 && xy.y == 0.0 {
            Angle::from(std::f64::NAN)
        } else {
            Angle::from(xy.y.atan2(xy.x))
        }
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HueAngle {
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
                if [DEG_0, DEG_120].contains(&abs_angle) {
                    0.0
                } else if [DEG_60, DEG_180].contains(&abs_angle) {
                    1.0
                } else {
                    fn f(angle: Angle) ->f64 {
                        angle.sin() / (DEG_120 - angle).sin()
                    };
                    if abs_angle <= DEG_60 {
                        f(abs_angle)
                    } else if abs_angle <= DEG_120 {
                        f(DEG_120 - abs_angle)
                    } else {
                        f(abs_angle - DEG_120)
                    }
                }
            }
            let other = calc_other(angle.abs());
            let max_chroma_rgb = if angle >= DEG_0 {
                if angle <= DEG_60 {
                    RGB::from((1.0, other, 0.0))
                } else if angle <= DEG_120 {
                    RGB::from((other, 1.0, 0.0))
                } else {
                    RGB::from((0.0, 1.0, other))
                }
            } else {
                if angle >= -DEG_60 {
                    RGB::from((1.0, 0.0, other))
                } else if angle >= -DEG_120 {
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
    pub fn is_grey(&self) -> bool {
        self.angle.is_nan()
    }

    pub fn get_angle(&self) -> Angle {
        self.angle
    }

    pub fn get_mac_chroma_rgb(&self) -> RGB {
        self.max_chroma_rgb
    }

    pub fn get_chroma_correction(&self) -> f64 {
        self.chroma_correction
    }

    pub fn max_chroma_for_value(&self, value: f64) -> f64 {
        assert!(is_proportion!(value));
        if self.is_grey() {
            0.0
        } else {
            let mcv = self.max_chroma_rgb.value();
            if mcv > value {
                value / mcv
            } else {
                (1.0 - value) / (1.0 - mcv)
            }
        }
    }

    pub fn rgb_range_with_chroma(&self, req_chroma: f64) -> Option<(RGB, RGB)> {
        assert!(is_proportion!(req_chroma));
        if req_chroma == 0.0 {
            Some((BLACK, WHITE))
        } else if self.is_grey() {
            None
        } else if req_chroma == 1.0 {
            Some((self.max_chroma_rgb, self.max_chroma_rgb))
        } else {
            let darkest = self.max_chroma_rgb * req_chroma;
            let lightest = darkest + WHITE * (1.0 - req_chroma);
            Some((darkest, lightest))
        }
    }

    pub fn value_range_with_chroma(&self, req_chroma: f64) -> Option<(f64, f64)> {
        assert!(is_proportion!(req_chroma));
        if req_chroma == 0.0 {
            Some((0.0, 1.0))
        } else if self.is_grey() {
            None
        } else if req_chroma == 1.0 {
            let val = self.max_chroma_rgb.value();
            Some((val, val))
        } else {
            let darkest = self.max_chroma_rgb.value() * req_chroma;
            let lightest = darkest + (1.0 - req_chroma);
            Some((darkest, lightest))
        }
    }

    pub fn rgb_with_chroma_and_value(&self, req_chroma: f64, req_value: f64) -> Option<(RGB)> {
        assert!(is_proportion!(req_chroma));
        assert!(is_proportion!(req_value));
        if let Some((min_value, max_value)) = self.value_range_with_chroma(req_chroma) {
            if req_value < min_value || req_value > max_value {
                return None
            } else {
                return Some(self.max_chroma_rgb * req_chroma + WHITE * (req_value - min_value))
            }
        }
        None
    }

    pub fn max_chroma_rgb_with_value(&self, req_value: f64) -> RGB {
        assert!(is_proportion!(req_value));
        let mcv = self.max_chroma_rgb.value();
        if mcv == req_value {
            self.max_chroma_rgb
        } else if mcv > req_value {
            if req_value == 0.0 {
                BLACK
            } else {
                self.max_chroma_rgb * req_value / mcv
            }
        } else if req_value == 1.0 {
            WHITE
        } else {
            let mut result = [1.0, 1.0, 1.0];
            let io = self.max_chroma_rgb.indices_value_order();
            // it's simpler two work out the weakest component first
            let other = self.max_chroma_rgb[io.1];
            let shortfall = (req_value - mcv) * 3.0;
            result[io.2] = shortfall / (2.0 - other);
            result[io.1] = other + shortfall - result[io.2];
            RGB::from(result)
        }
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

        fn calculate_chroma(self) -> f64 {
            XY::from(self).calculate_chroma()
        }
    }

    #[test]
    fn rgb_math_hue_xy_components()  {
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
    fn rgb_math_hue_xy_to_angle() {
        assert!(Angle::from(XY{x: 0.0, y: 0.0}).is_nan());
        assert_eq!(Angle::from(XY{x: 1.0, y: 0.0}), DEG_0);
        assert_eq!(Angle::from(XY{x: -1.0, y: 0.0}), DEG_180);
        assert_eq!(Angle::from(XY{x: 1.0, y: 1.0}), DEG_45);
        assert_eq!(Angle::from(XY{x: -1.0, y: -1.0}), DEG_180 + DEG_45 );
        assert_eq!(Angle::from(XY{x: 0.0, y: 1.0}), DEG_90);
        assert_eq!(Angle::from(XY{x: 0.0, y: -1.0}), -DEG_90);
    }

    #[test]
    fn rgb_math_hue_angle_basics() {
        assert_eq!(HueAngle::from(DEG_0).max_chroma_rgb, RED);
        assert_eq!(HueAngle::from(DEG_60).max_chroma_rgb, YELLOW);
        assert_eq!(HueAngle::from(DEG_120).max_chroma_rgb, GREEN);
        assert_eq!(HueAngle::from(DEG_180).max_chroma_rgb, CYAN);
        assert_eq!(HueAngle::from(-DEG_0).max_chroma_rgb, RED);
        assert_eq!(HueAngle::from(-DEG_60).max_chroma_rgb, MAGENTA);
        assert_eq!(HueAngle::from(-DEG_120).max_chroma_rgb, BLUE);
        assert_eq!(HueAngle::from(-DEG_180).max_chroma_rgb, CYAN);

        assert_eq!(HueAngle::from(DEG_60) - DEG_60, HueAngle::from(DEG_0));
        assert_eq!(HueAngle::from(DEG_60) + DEG_60, HueAngle::from(DEG_120));

        assert!(within_limit((HueAngle::from(DEG_120) - HueAngle::from(DEG_90)).radians(), DEG_30.radians()));

        for angle in [DEG_0, DEG_60, DEG_120, DEG_180, -DEG_0, -DEG_60, -DEG_120, -DEG_180].iter() {
            assert_eq!(HueAngle::from(*angle).chroma_correction, 1.0);
        };
        for mul in 1..7 {
            let hue_angle = HueAngle::from(DEG_30 * mul);
            assert!(within_limit(hue_angle.max_chroma_rgb.calculate_chroma(), 1.0));
        };
        for angle in [DEG_30, DEG_90].iter() {
            assert_eq!(HueAngle::from(*angle).max_chroma_rgb.non_zero_indices(), vec![0, 1])
        };
        for angle in [-DEG_30, -DEG_90].iter() {
            assert_eq!(HueAngle::from(*angle).max_chroma_rgb.non_zero_indices(), vec![0, 2])
        };
        for angle in [DEG_150, -DEG_150].iter() {
            assert_eq!(HueAngle::from(*angle).max_chroma_rgb.non_zero_indices(), vec![1, 2])
        };
        assert_eq!(HueAngle::from(DEG_30).max_chroma_rgb.indices_value_order(), (0, 1, 2));
        assert_eq!(HueAngle::from(DEG_90).max_chroma_rgb.indices_value_order(), (1, 0, 2));
        assert_eq!(HueAngle::from(DEG_150).max_chroma_rgb.indices_value_order(), (1, 2, 0));
        assert_eq!(HueAngle::from(-DEG_30).max_chroma_rgb.indices_value_order(), (0, 2, 1));
        assert_eq!(HueAngle::from(-DEG_90).max_chroma_rgb.indices_value_order(), (2, 0, 1));
        assert_eq!(HueAngle::from(-DEG_150).max_chroma_rgb.indices_value_order(), (2, 1, 0));
        for angle in [DEG_30, DEG_90, DEG_150, -DEG_30, -DEG_90, -DEG_150].iter() {
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
            assert!(HueAngle::from(*rgb).is_grey());
        }
    }

    #[test]
    fn rgb_math_hue_angle_max_chroma_for_value() {
        for angle in [DEG_0, DEG_30, DEG_60, DEG_90, DEG_120, DEG_150, DEG_180, -DEG_0, -DEG_30 -DEG_60, -DEG_90, -DEG_120, -DEG_150, -DEG_180].iter() {
            let hue_angle = HueAngle::from(*angle);
            for value in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
                let max_chroma = hue_angle.max_chroma_for_value(*value);
                if let Some(rgb) = hue_angle.rgb_with_chroma_and_value(max_chroma, *value) {
                    assert!(within_limit(rgb.calculate_chroma(), max_chroma));
                    assert!(within_limit(rgb.value(), *value));
                    assert!((hue_angle - HueAngle::from(rgb)).abs().radians() <= 0.00000000001);
                } else {
                    assert!(false)
                };
                if let Some((min_value, max_value)) = hue_angle.value_range_with_chroma(max_chroma) {
                    assert!(within_limit_quiet(min_value, *value) || within_limit_quiet(max_value, *value));
                } else {
                    assert!(false)
                }
            };
            for value in [0.0, 1.0].iter() {
                let max_chroma = hue_angle.max_chroma_for_value(*value);
                let rgb = hue_angle.rgb_with_chroma_and_value(max_chroma, *value).unwrap();
                assert!(within_limit(rgb.calculate_chroma(), max_chroma));
                assert!(within_limit(rgb.value(), *value));
                assert!(HueAngle::from(rgb).is_grey());
            }
        }
        let hue_angle = HueAngle::from(XY::from((0.0, 0.0)));
        for value in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
            let max_chroma = hue_angle.max_chroma_for_value(*value);
            assert_eq!(max_chroma, 0.0);
            if let Some(rgb) = hue_angle.rgb_with_chroma_and_value(max_chroma, *value) {
                assert!(within_limit(rgb.calculate_chroma(), max_chroma));
                assert!(within_limit(rgb.value(), *value));
                assert!(HueAngle::from(rgb).is_grey());
            } else {
                assert!(false)
            };
        }
    }

    #[test]
    fn rgb_math_hue_rgb_range_with_chroma() {
        for angle in [DEG_0, DEG_30, DEG_60, DEG_90, DEG_120, DEG_150, DEG_180, -DEG_0, -DEG_30 -DEG_60, -DEG_90, -DEG_120, -DEG_150, -DEG_180].iter() {
            let hue_angle = HueAngle::from(*angle);
            assert_eq!(hue_angle.rgb_range_with_chroma(0.0).unwrap(), (BLACK, WHITE));
            for chroma in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                let (rgb_shade, rgb_tint) = hue_angle.rgb_range_with_chroma(*chroma).unwrap();
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
        let hue_angle = HueAngle::from(XY::from((0.0, 0.0)));
        assert_eq!(hue_angle.rgb_range_with_chroma(0.0).unwrap(), (BLACK, WHITE));
        for chroma in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
            assert_eq!(hue_angle.rgb_range_with_chroma(*chroma), None);
            assert_eq!(hue_angle.value_range_with_chroma(*chroma), None);
        }
    }

    #[test]
    fn rgb_math_hue_rgb_with_chroma_and_value_extremities() {
        for angle in [DEG_0, DEG_30, DEG_60, DEG_90, DEG_120, DEG_150, DEG_180, -DEG_0, -DEG_30 -DEG_60, -DEG_90, -DEG_120, -DEG_150, -DEG_180].iter() {
            let hue_angle = HueAngle::from(*angle);
            for chroma in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                if let Some((min_value, max_value)) = hue_angle.value_range_with_chroma(*chroma) {
                    let rgb_shade = hue_angle.rgb_with_chroma_and_value(*chroma, min_value).unwrap();
                    let rgb_tint = hue_angle.rgb_with_chroma_and_value(*chroma, max_value).unwrap();
                    let shade_chroma = rgb_shade.calculate_chroma();
                    let tint_chroma = rgb_tint.calculate_chroma();
                    assert!(within_limit(shade_chroma, *chroma));
                    assert!(within_limit(tint_chroma, *chroma));
                    let shade_angle = HueAngle::from(rgb_shade);
                    let tint_angle = HueAngle::from(rgb_tint);
                    assert!((hue_angle - shade_angle).abs().radians() <= 0.00000000001);
                    assert!((hue_angle - tint_angle).abs().radians() <= 0.00000000001);
                } else {
                    assert!(false) //should not get here for real angles
                }
            }
        }
    }

    #[test]
    fn rgb_math_hue_rgb_with_chroma_and_value() {
        for angle in [DEG_0, DEG_30, DEG_60, DEG_90, DEG_120, DEG_150, DEG_180, -DEG_0, -DEG_30 -DEG_60, -DEG_90, -DEG_120, -DEG_150, -DEG_180].iter() {
            let hue_angle = HueAngle::from(*angle);
            for chroma in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                for value in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                    match hue_angle.rgb_with_chroma_and_value(*chroma, *value) {
                        Some(rgb) => {
                            assert!(within_limit(rgb.calculate_chroma(), *chroma));
                            assert!(within_limit(rgb.value(), *value));
                            assert!((hue_angle - HueAngle::from(rgb)).abs().radians() <= 0.00000000001);
                        },
                        None => (
                            if let Some((min_value, max_value)) = hue_angle.value_range_with_chroma(*chroma) {
                                assert!(*value < min_value || *value > max_value);
                            }
                        )
                    }
                }
            }
            for value in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                match hue_angle.rgb_with_chroma_and_value(0.0, *value) {
                    Some(rgb) => {
                        assert!(within_limit(rgb.calculate_chroma(), 0.0));
                        assert!(within_limit(rgb.value(), *value));
                        assert!(HueAngle::from(rgb).is_grey());
                    },
                    None => (
                        assert!(false)
                    )
                }
            }
        }
        let hue_angle = HueAngle::from(XY::from((0.0, 0.0)));
        for chroma in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
            for value in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                assert_eq!(hue_angle.rgb_with_chroma_and_value(*chroma, *value), None);
            }
        }
        for value in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
            assert_eq!(hue_angle.rgb_with_chroma_and_value(0.0, *value), Some(WHITE * *value));
        }
    }

    #[test]
    fn rgb_math_hue_max_chroma_rgb_for_value() {
        for angle in [DEG_0, DEG_30, DEG_60, DEG_90, DEG_120, DEG_150, DEG_180, -DEG_0, -DEG_30 -DEG_60, -DEG_90, -DEG_120, -DEG_150, -DEG_180].iter() {
            let hue_angle = HueAngle::from(*angle);
            for value in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.99].iter() {
                let rgb = hue_angle.max_chroma_rgb_with_value(*value);
                assert!(within_limit(rgb.value(), *value));
                assert!((hue_angle - HueAngle::from(rgb)).abs().radians() <= 0.00000000001);
                let max_chroma = hue_angle.max_chroma_for_value(*value);
                assert!(within_limit(rgb.calculate_chroma(), max_chroma));
            }
            for value in [0.0, 1.0].iter() {
                let rgb = hue_angle.max_chroma_rgb_with_value(*value);
                assert!(within_limit(rgb.value(), *value));
                assert!(HueAngle::from(rgb).is_grey());
            }
        }
        let hue_angle = HueAngle::from(XY::from((0.0, 0.0)));
        for value in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
            assert_eq!(hue_angle.max_chroma_rgb_with_value(*value), WHITE * *value);
        }
    }
}