// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std;
use std::cmp::{Ordering, PartialOrd};
use std::convert::{From, TryFrom};
use std::hash::*;
use std::ops::{Add, Sub};

use crate::rgb_math::angle::*;
use crate::rgb_math::rgb::*;

macro_rules! is_proportion {
    ( $x:expr ) => {{
        ($x <= 1.0) && ($x >= 0.0)
    }};
}

const SIN_120: f64 = 0.86602_54037_844387;
const COS_120: f64 = -0.5;

macro_rules! rgb_x_coord {
    ( $rgb:expr ) => {{
        $rgb.red + ($rgb.green + $rgb.blue) * COS_120
    }};
}

macro_rules! rgb_y_coord {
    ( $rgb:expr ) => {{
        ($rgb.green - $rgb.blue) * SIN_120
    }};
}

pub trait XYHA {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn xy(&self) -> (f64, f64);
    fn hypot(&self) -> f64;
    fn angle(&self) -> Option<Angle>;
}

impl XYHA for RGB {
    fn x(&self) -> f64 {
        rgb_x_coord!(self)
    }

    fn y(&self) -> f64 {
        rgb_y_coord!(self)
    }

    fn xy(&self) -> (f64, f64) {
        (rgb_x_coord!(self), rgb_y_coord!(self))
    }

    fn hypot(&self) -> f64 {
        // Be paranoid about fact floats only approximate reals
        rgb_x_coord!(self).hypot(rgb_y_coord!(self)).min(1.0)
    }

    fn angle(&self) -> Option<Angle> {
        let x = rgb_x_coord!(self);
        let y = rgb_y_coord!(self);
        if let Ok(angle) = Angle::try_from((x, y)) {
            Some(angle)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HueAngle {
    angle: Angle,
    max_chroma_rgb: RGB,
    chroma_correction: f64,
}

impl Hash for HueAngle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.angle.radians().to_bits());
    }
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

fn calc_other(abs_angle: Angle) -> f64 {
    if [Angle::DEG_0, Angle::DEG_120].contains(&abs_angle) {
        0.0
    } else if [Angle::DEG_60, Angle::DEG_180].contains(&abs_angle) {
        1.0
    } else {
        fn f(angle: Angle) -> f64 {
            // Careful of float not fully representing reals
            (angle.sin() / (Angle::DEG_120 - angle).sin()).min(1.0)
        };
        if abs_angle <= Angle::DEG_60 {
            f(abs_angle)
        } else if abs_angle <= Angle::DEG_120 {
            f(Angle::DEG_120 - abs_angle)
        } else {
            f(abs_angle - Angle::DEG_120)
        }
    }
}

impl From<Angle> for HueAngle {
    fn from(angle: Angle) -> HueAngle {
        let other = calc_other(angle.abs());
        let max_chroma_rgb = if angle >= Angle::DEG_0 {
            if angle <= Angle::DEG_60 {
                RGB::from((1.0, other, 0.0))
            } else if angle <= Angle::DEG_120 {
                RGB::from((other, 1.0, 0.0))
            } else {
                RGB::from((0.0, 1.0, other))
            }
        } else {
            if angle >= -Angle::DEG_60 {
                RGB::from((1.0, 0.0, other))
            } else if angle >= -Angle::DEG_120 {
                RGB::from((other, 0.0, 1.0))
            } else {
                RGB::from((0.0, other, 1.0))
            }
        };
        // Careful of float not fully representing reals
        let chroma_correction = (1.0 + other * other - other).sqrt().min(1.0).recip();
        HueAngle {
            angle,
            max_chroma_rgb,
            chroma_correction,
        }
    }
}

impl TryFrom<RGB> for HueAngle {
    type Error = &'static str;
    fn try_from(rgb: RGB) -> Result<HueAngle, Self::Error> {
        use std::convert::TryInto;
        let angle: Angle = rgb.xy().try_into()?;
        let io = rgb.indices_value_order();
        // Careful of float not fully representing reals
        let mut parts: [f64; 3] = [0.0, 0.0, 0.0];
        parts[io.0] = 1.0;
        if rgb[io.0] == rgb[io.1] {
            // SECONDARY
            parts[io.1] = 1.0;
        } else if rgb[io.1] != rgb[io.2] {
            // NOT PRIMARY or SECONDARY
            parts[io.1] = calc_other(angle.abs());
        }
        let max_chroma_rgb = RGB::from(parts);
        let chroma_correction = max_chroma_rgb.hypot().recip();
        Ok(HueAngle {
            angle,
            max_chroma_rgb,
            chroma_correction,
        })
    }
}

impl HueAngle {
    pub fn angle(&self) -> Angle {
        self.angle
    }

    pub fn max_chroma_rgb(&self) -> RGB {
        self.max_chroma_rgb
    }

    pub fn chroma_correction(&self) -> f64 {
        self.chroma_correction
    }

    pub fn max_chroma_for_value(&self, value: f64) -> f64 {
        assert!(is_proportion!(value));
        let mcv = self.max_chroma_rgb.value();
        if mcv > value {
            value / mcv
        } else {
            (1.0 - value) / (1.0 - mcv)
        }
    }

    pub fn rgb_range_with_chroma(&self, req_chroma: f64) -> (RGB, RGB) {
        assert!(is_proportion!(req_chroma));
        if req_chroma == 0.0 {
            (BLACK, WHITE)
        } else if req_chroma == 1.0 {
            (self.max_chroma_rgb, self.max_chroma_rgb)
        } else {
            let darkest = self.max_chroma_rgb * req_chroma;
            let lightest = darkest + WHITE * (1.0 - req_chroma);
            (darkest, lightest)
        }
    }

    pub fn value_range_with_chroma(&self, req_chroma: f64) -> (f64, f64) {
        assert!(is_proportion!(req_chroma));
        if req_chroma == 0.0 {
            (0.0, 1.0)
        } else if req_chroma == 1.0 {
            let val = self.max_chroma_rgb.value();
            (val, val)
        } else {
            let darkest = self.max_chroma_rgb.value() * req_chroma;
            let lightest = darkest + (1.0 - req_chroma);
            (darkest, lightest)
        }
    }

    pub fn rgb_with_chroma_and_value(&self, req_chroma: f64, req_value: f64) -> Option<(RGB)> {
        assert!(is_proportion!(req_chroma));
        assert!(is_proportion!(req_value));
        let (min_value, max_value) = self.value_range_with_chroma(req_chroma);
        if req_value < min_value || req_value > max_value {
            None
        } else {
            // NB: because floats only approximate reals trying to
            // set chroma too small (but non zero) results in a drift
            // in the hue angle of the resulting RGB. When this
            // happens we go straight to a zero chroma RGB
            let rgb = self.max_chroma_rgb * req_chroma + WHITE * (req_value - min_value);
            if let Some(rgb_angle) = rgb.angle() {
                if (rgb_angle - self.angle).abs().radians() < 0.00001 {
                    Some(rgb)
                } else {
                    Some(WHITE * req_value)
                }
            } else {
                Some(WHITE * req_value)
            }
        }
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

    fn within_limit_quiet(x1: f64, x2: f64) -> bool {
        let limit = 0.0000000001;
        if x1 == 0.0 || x2 == 0.0 {
            (x2 + x1).abs() < limit
        } else {
            ((x1 / x2) - 1.0).abs() < limit
        }
    }

    fn within_limit(x1: f64, x2: f64) -> bool {
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
                return false;
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
            }
            v
        }

        fn calculate_chroma(self) -> f64 {
            if let Ok(angle) = HueAngle::try_from(self) {
                self.hypot() * angle.chroma_correction
            } else {
                0.0
            }
        }
    }

    #[test]
    fn rgb_math_hue_angle_basics() {
        assert_eq!(HueAngle::from(Angle::DEG_0).max_chroma_rgb, RED);
        assert_eq!(HueAngle::from(Angle::DEG_60).max_chroma_rgb, YELLOW);
        assert_eq!(HueAngle::from(Angle::DEG_120).max_chroma_rgb, GREEN);
        assert_eq!(HueAngle::from(Angle::DEG_180).max_chroma_rgb, CYAN);
        assert_eq!(HueAngle::from(-Angle::DEG_0).max_chroma_rgb, RED);
        assert_eq!(HueAngle::from(-Angle::DEG_60).max_chroma_rgb, MAGENTA);
        assert_eq!(HueAngle::from(-Angle::DEG_120).max_chroma_rgb, BLUE);
        assert_eq!(HueAngle::from(-Angle::DEG_180).max_chroma_rgb, CYAN);

        assert_eq!(
            HueAngle::from(Angle::DEG_60) - Angle::DEG_60,
            HueAngle::from(Angle::DEG_0)
        );
        assert_eq!(
            HueAngle::from(Angle::DEG_60) + Angle::DEG_60,
            HueAngle::from(Angle::DEG_120)
        );

        assert!(within_limit(
            (HueAngle::from(Angle::DEG_120) - HueAngle::from(Angle::DEG_90)).radians(),
            Angle::DEG_30.radians()
        ));

        for angle in [
            Angle::DEG_0,
            Angle::DEG_60,
            Angle::DEG_120,
            Angle::DEG_180,
            -Angle::DEG_0,
            -Angle::DEG_60,
            -Angle::DEG_120,
            -Angle::DEG_180,
        ]
        .iter()
        {
            assert_eq!(HueAngle::from(*angle).chroma_correction, 1.0);
        }
        for mul in 1..7 {
            let hue_angle = HueAngle::from(Angle::DEG_30 * mul);
            assert!(within_limit(
                hue_angle.max_chroma_rgb.calculate_chroma(),
                1.0
            ));
        }
        for angle in [Angle::DEG_30, Angle::DEG_90].iter() {
            assert_eq!(
                HueAngle::from(*angle).max_chroma_rgb.non_zero_indices(),
                vec![0, 1]
            )
        }
        for angle in [-Angle::DEG_30, -Angle::DEG_90].iter() {
            assert_eq!(
                HueAngle::from(*angle).max_chroma_rgb.non_zero_indices(),
                vec![0, 2]
            )
        }
        for angle in [Angle::DEG_150, -Angle::DEG_150].iter() {
            assert_eq!(
                HueAngle::from(*angle).max_chroma_rgb.non_zero_indices(),
                vec![1, 2]
            )
        }
        assert_eq!(
            HueAngle::from(Angle::DEG_30)
                .max_chroma_rgb
                .indices_value_order(),
            (0, 1, 2)
        );
        assert_eq!(
            HueAngle::from(Angle::DEG_90)
                .max_chroma_rgb
                .indices_value_order(),
            (1, 0, 2)
        );
        assert_eq!(
            HueAngle::from(Angle::DEG_150)
                .max_chroma_rgb
                .indices_value_order(),
            (1, 2, 0)
        );
        assert_eq!(
            HueAngle::from(-Angle::DEG_30)
                .max_chroma_rgb
                .indices_value_order(),
            (0, 2, 1)
        );
        assert_eq!(
            HueAngle::from(-Angle::DEG_90)
                .max_chroma_rgb
                .indices_value_order(),
            (2, 0, 1)
        );
        assert_eq!(
            HueAngle::from(-Angle::DEG_150)
                .max_chroma_rgb
                .indices_value_order(),
            (2, 1, 0)
        );
        for angle in [
            Angle::DEG_30,
            Angle::DEG_90,
            Angle::DEG_150,
            -Angle::DEG_30,
            -Angle::DEG_90,
            -Angle::DEG_150,
        ]
        .iter()
        {
            let hue_angle = HueAngle::from(*angle);
            let second_index = hue_angle.max_chroma_rgb.indices_value_order().1;
            assert!(within_limit(hue_angle.max_chroma_rgb[second_index], 0.5));
        }
        for rgb in [RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA].iter() {
            assert!(within_limits(
                HueAngle::try_from(*rgb).unwrap().max_chroma_rgb,
                *rgb
            ));
            for m in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
                assert!(within_limits(
                    HueAngle::try_from(*rgb * *m).unwrap().max_chroma_rgb,
                    *rgb
                ));
                let tint = (*rgb + WHITE) * 0.5;
                assert!(within_limits(
                    HueAngle::try_from(tint * *m).unwrap().max_chroma_rgb,
                    *rgb
                ));
            }
        }
        for g in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
            let rgb = RGB::from((1.0, *g, 0.0));
            assert!(within_limits(
                HueAngle::try_from(rgb).unwrap().max_chroma_rgb,
                rgb
            ));
            for m in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
                assert!(within_limits(
                    HueAngle::try_from(rgb * *m).unwrap().max_chroma_rgb,
                    rgb
                ));
                let tint = (rgb + WHITE) * 0.5;
                assert!(within_limits(
                    HueAngle::try_from(tint * *m).unwrap().max_chroma_rgb,
                    rgb
                ));
            }
        }
        for rgb in [BLACK, WHITE, WHITE * 0.5].iter() {
            assert!(HueAngle::try_from(*rgb).is_err());
        }
    }

    #[test]
    fn rgb_math_hue_angle_max_chroma_for_value() {
        for angle in [
            Angle::DEG_0,
            Angle::DEG_30,
            Angle::DEG_60,
            Angle::DEG_90,
            Angle::DEG_120,
            Angle::DEG_150,
            Angle::DEG_180,
            -Angle::DEG_0,
            -Angle::DEG_30,
            -Angle::DEG_60,
            -Angle::DEG_90,
            -Angle::DEG_120,
            -Angle::DEG_150,
            -Angle::DEG_180,
        ]
        .iter()
        {
            let hue_angle = HueAngle::from(*angle);
            for value in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
                let max_chroma = hue_angle.max_chroma_for_value(*value);
                if let Some(rgb) = hue_angle.rgb_with_chroma_and_value(max_chroma, *value) {
                    assert!(within_limit(rgb.calculate_chroma(), max_chroma));
                    assert!(within_limit(rgb.value(), *value));
                    assert!(
                        (hue_angle - HueAngle::try_from(rgb).unwrap())
                            .abs()
                            .radians()
                            <= 0.00000000001
                    );
                } else {
                    assert!(false)
                };
                let (min_value, max_value) = hue_angle.value_range_with_chroma(max_chroma);
                assert!(
                    within_limit_quiet(min_value, *value) || within_limit_quiet(max_value, *value)
                );
            }
            for value in [0.0, 1.0].iter() {
                let max_chroma = hue_angle.max_chroma_for_value(*value);
                let rgb = hue_angle
                    .rgb_with_chroma_and_value(max_chroma, *value)
                    .unwrap();
                assert!(within_limit(rgb.calculate_chroma(), max_chroma));
                assert!(within_limit(rgb.value(), *value));
                assert!(HueAngle::try_from(rgb).is_err());
            }
        }
    }

    #[test]
    fn rgb_math_hue_rgb_range_with_chroma() {
        for angle in [
            Angle::DEG_0,
            Angle::DEG_30,
            Angle::DEG_60,
            Angle::DEG_90,
            Angle::DEG_120,
            Angle::DEG_150,
            Angle::DEG_180,
            -Angle::DEG_0,
            -Angle::DEG_30,
            -Angle::DEG_60,
            -Angle::DEG_90,
            -Angle::DEG_120,
            -Angle::DEG_150,
            -Angle::DEG_180,
        ]
        .iter()
        {
            let hue_angle = HueAngle::from(*angle);
            assert_eq!(hue_angle.rgb_range_with_chroma(0.0), (BLACK, WHITE));
            for chroma in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                let (rgb_shade, rgb_tint) = hue_angle.rgb_range_with_chroma(*chroma);
                assert!(rgb_shade.value() <= rgb_tint.value());
                let shade_chroma = rgb_shade.calculate_chroma();
                let tint_chroma = rgb_tint.calculate_chroma();
                assert!(within_limit(shade_chroma, *chroma));
                assert!(within_limit(tint_chroma, *chroma));
                let shade_angle = HueAngle::try_from(rgb_shade).unwrap();
                let tint_angle = HueAngle::try_from(rgb_tint).unwrap();
                assert!((hue_angle - shade_angle).abs().radians() <= 0.00000000001);
                assert!((hue_angle - tint_angle).abs().radians() <= 0.00000000001);
            }
        }
    }

    #[test]
    fn rgb_math_hue_rgb_with_chroma_and_value_extremities() {
        for angle in [
            Angle::DEG_0,
            Angle::DEG_30,
            Angle::DEG_60,
            Angle::DEG_90,
            Angle::DEG_120,
            Angle::DEG_150,
            Angle::DEG_180,
            -Angle::DEG_0,
            -Angle::DEG_30,
            -Angle::DEG_60,
            -Angle::DEG_90,
            -Angle::DEG_120,
            -Angle::DEG_150,
            -Angle::DEG_180,
        ]
        .iter()
        {
            let hue_angle = HueAngle::from(*angle);
            for chroma in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                let (min_value, max_value) = hue_angle.value_range_with_chroma(*chroma);
                let rgb_shade = hue_angle
                    .rgb_with_chroma_and_value(*chroma, min_value)
                    .unwrap();
                let rgb_tint = hue_angle
                    .rgb_with_chroma_and_value(*chroma, max_value)
                    .unwrap();
                let shade_chroma = rgb_shade.calculate_chroma();
                let tint_chroma = rgb_tint.calculate_chroma();
                assert!(within_limit(shade_chroma, *chroma));
                assert!(within_limit(tint_chroma, *chroma));
                let shade_angle = HueAngle::try_from(rgb_shade).unwrap();
                let tint_angle = HueAngle::try_from(rgb_tint).unwrap();
                assert!((hue_angle - shade_angle).abs().radians() <= 0.00000000001);
                assert!((hue_angle - tint_angle).abs().radians() <= 0.00000000001);
            }
        }
    }

    #[test]
    fn rgb_math_hue_rgb_with_chroma_and_value() {
        let mut count_a = 0;
        let mut count_b = 0;
        for angle in [
            Angle::DEG_0,
            Angle::DEG_30,
            Angle::DEG_60,
            Angle::DEG_90,
            Angle::DEG_120,
            Angle::DEG_150,
            Angle::DEG_180,
            -Angle::DEG_0,
            -Angle::DEG_30,
            -Angle::DEG_60,
            -Angle::DEG_90,
            -Angle::DEG_120,
            -Angle::DEG_150,
            -Angle::DEG_180,
        ]
        .iter()
        {
            let hue_angle = HueAngle::from(*angle);
            for chroma in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                for value in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                    match hue_angle.rgb_with_chroma_and_value(*chroma, *value) {
                        Some(rgb) => {
                            assert!(within_limit(rgb.calculate_chroma(), *chroma));
                            assert!(within_limit(rgb.value(), *value));
                            assert!(
                                (hue_angle - HueAngle::try_from(rgb).unwrap())
                                    .abs()
                                    .radians()
                                    <= 0.00000001
                            );
                        }
                        None => {
                            let (min_value, max_value) = hue_angle.value_range_with_chroma(*chroma);
                            assert!(*value < min_value || *value > max_value);
                        }
                    }
                }
            }
            // check for handling of hue drift for small chroma values
            for value in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                for chroma in [0.000000001, 0.0000000001, 0.00000000001, 0.000000000001].iter() {
                    match hue_angle.rgb_with_chroma_and_value(*chroma, *value) {
                        Some(rgb) => {
                            if rgb.angle().is_none() {
                                assert!(within_limit(rgb.value(), *value));
                                count_a += 1;
                            } else {
                                assert!(within_limit(rgb.value(), *value));
                                assert!(
                                    (hue_angle - HueAngle::try_from(rgb).unwrap())
                                        .abs()
                                        .radians()
                                        <= 0.00001
                                );
                                count_b += 1;
                            };
                        }
                        None => {
                            let (min_value, max_value) = hue_angle.value_range_with_chroma(*chroma);
                            assert!(*value < min_value || *value > max_value);
                        }
                    }
                }
            }
            for value in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                match hue_angle.rgb_with_chroma_and_value(0.0, *value) {
                    Some(rgb) => {
                        assert!(within_limit(rgb.calculate_chroma(), 0.0));
                        assert!(within_limit(rgb.value(), *value));
                        assert!(HueAngle::try_from(rgb).is_err());
                    }
                    None => (assert!(false)),
                }
            }
        }
        assert!(count_a > 0);
        assert!(count_b > 0);
    }

    #[test]
    fn rgb_math_hue_max_chroma_rgb_for_value() {
        for angle in [
            Angle::DEG_0,
            Angle::DEG_30,
            Angle::DEG_60,
            Angle::DEG_90,
            Angle::DEG_120,
            Angle::DEG_150,
            Angle::DEG_180,
            -Angle::DEG_0,
            -Angle::DEG_30,
            -Angle::DEG_60,
            -Angle::DEG_90,
            -Angle::DEG_120,
            -Angle::DEG_150,
            -Angle::DEG_180,
        ]
        .iter()
        {
            let hue_angle = HueAngle::from(*angle);
            for value in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.99].iter() {
                let rgb = hue_angle.max_chroma_rgb_with_value(*value);
                assert!(within_limit(rgb.value(), *value));
                assert!(
                    (hue_angle - HueAngle::try_from(rgb).unwrap())
                        .abs()
                        .radians()
                        <= 0.00000000001
                );
                let max_chroma = hue_angle.max_chroma_for_value(*value);
                assert!(within_limit(rgb.calculate_chroma(), max_chroma));
            }
            for value in [0.0, 1.0].iter() {
                let rgb = hue_angle.max_chroma_rgb_with_value(*value);
                assert!(within_limit(rgb.value(), *value));
                assert!(HueAngle::try_from(rgb).is_err());
            }
        }
    }
}
