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

use std::cmp::{Ordering, PartialOrd};
use std::convert::From;
use std::f64::consts;
use std::ops::{Div, Mul, Add, Sub, Neg};

#[macro_export]
macro_rules! is_normalised {
    ( $x:expr ) => {
        {
            ($x <= consts::PI) && ($x >= -consts::PI)
        }
    }
}

#[macro_export]
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
pub struct Angle (f64);

pub const DEG_0: Angle   = Angle(0.0);
pub const DEG_5: Angle   = Angle(consts::FRAC_PI_6 / 6.0);
pub const DEG_10: Angle  = Angle(consts::FRAC_PI_6 / 3.0);
pub const DEG_30: Angle  = Angle(consts::FRAC_PI_6);
pub const DEG_45: Angle  = Angle(consts::FRAC_PI_4);
pub const DEG_60: Angle  = Angle(consts::FRAC_PI_3);
pub const DEG_90: Angle  = Angle(consts::FRAC_PI_2);
pub const DEG_120: Angle = Angle(consts::FRAC_PI_3 * 2.0);
pub const DEG_150: Angle = Angle(consts::FRAC_PI_6 * 5.0);
pub const DEG_180: Angle = Angle(consts::PI);

impl From<f64> for Angle {
    fn from(f: f64) -> Angle {
        if f.is_nan() {
            Angle(f)
        } else {
            Angle(normalise!(f))
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

#[cfg(test)]
mod tests {
    use super::*;

    const ANGLES: [Angle; 10] = [DEG_0, DEG_5, DEG_10, DEG_30, DEG_45, DEG_60, DEG_90, DEG_120, DEG_150, DEG_180];

    fn within_limit_quiet(x1: Angle, x2:Angle) -> bool {
        let limit = 0.0000000001;
        if x1.radians() == 0.0 || x2.radians() == 0.0 {
            (x2.radians() + x1.radians()).abs() < limit
        } else {
            ((x1.radians() / x2.radians()) - 1.0).abs() < limit
        }
    }

    fn within_limit(x1: Angle, x2:Angle) -> bool {
        if within_limit_quiet(x1, x2) {
            true
        } else {
            println!("{:?} != {:?}", x1.degrees(), x2.degrees());
            false
        }
    }

    #[test]
    fn rgb_math_angle_constants() {
        for angle in ANGLES.iter() {
            assert!(is_normalised!((*angle).0))
        }
    }

    #[test]
    fn rgb_math_angle_sub() {
        for angle in ANGLES.iter() {
            assert_eq!((*angle - *angle), DEG_0)
        }
        assert!(within_limit(-DEG_150 - DEG_150, DEG_60));
        assert!(within_limit(DEG_150 - -DEG_150, -DEG_60));
        assert!(within_limit(DEG_150 - -DEG_180, -DEG_30));
        assert!(within_limit(DEG_150 - DEG_180, -DEG_30));
    }

    #[test]
    fn rgb_math_angle_add() {
        for angle in ANGLES.iter() {
            assert_eq!((*angle - *angle), DEG_0)
        }
        assert!(within_limit(DEG_150 + DEG_60, -DEG_150));
        assert!(within_limit(-DEG_150 + DEG_60, -DEG_90));
        assert!(within_limit(DEG_150 + DEG_180, -DEG_30));
        assert!(within_limit(DEG_150 + -DEG_180, -DEG_30));
    }

    #[test]
    fn rgb_math_angle_cmp() {
        assert!(DEG_30 > DEG_5);
        assert!(-DEG_150 > DEG_150);
        assert!(-DEG_150 < DEG_10);
    }
}
