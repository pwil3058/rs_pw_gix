// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::rgb_math::angle::*;
use crate::rgb_math::hue::*;
use crate::rgb_math::rgb::*;

pub struct RGBManipulator {
    rgb: RGB,
    angle: Option<HueAngle>,
    last_angle: Option<HueAngle>,
    chroma: f64,
}

impl RGBManipulator {
    pub fn new() -> RGBManipulator {
        RGBManipulator {
            rgb: WHITE,
            angle: None,
            last_angle: None,
            chroma: 0.0,
        }
    }

    pub fn set_rgb(&mut self, rgb: RGB) {
        self.rgb = rgb;
        use std::convert::TryFrom;
        if let Ok(new_angle) = HueAngle::try_from(rgb) {
            // Be paranoid about fact floats only approximate reals
            self.chroma = (rgb.hypot() * new_angle.chroma_correction()).min(1.0);
            self.last_angle = Some(new_angle);
            self.angle = Some(new_angle);
        } else {
            self.chroma = 0.0;
            self.angle = None;
        }
    }

    pub fn get_rgb(&self) -> RGB {
        self.rgb
    }

    pub fn decr_chroma(&mut self, delta: f64) -> bool {
        assert!(is_proportion!(delta));
        if let Some(angle) = self.angle {
            let cur_chroma = self.chroma;
            let cur_value = self.rgb.value();
            let new_chroma = (cur_chroma - delta).max(0.0);
            let rgbe = angle.rgb_with_chroma_and_value(new_chroma, cur_value);
            if let Some(new_rgb) = rgbe {
                self.set_rgb(new_rgb);
                // NB: beware frailties of float versus real
                self.chroma != cur_chroma
            } else {
                panic!("File: {:?} Line: {:?}", file!(), line!())
            }
        } else {
            false
        }
    }

    pub fn incr_chroma(&mut self, delta: f64) -> bool {
        assert!(is_proportion!(delta));
        let cur_value = self.rgb.value();
        let viable_angle = if self.angle.is_none() {
            if let Some(last_angle) = self.last_angle {
                Some(last_angle)
            } else {
                Some(HueAngle::from(Angle::DEG_0))
            }
        } else {
            self.angle
        };
        if let Some(viable_angle) = viable_angle {
            let max_chroma = viable_angle.max_chroma_for_value(cur_value).min(1.0);
            let cur_chroma = self.chroma;
            let new_chroma = (cur_chroma + delta).min(1.0);
            if new_chroma >= max_chroma {
                self.set_rgb(viable_angle.max_chroma_rgb_with_value(cur_value));
                // NB: beware frailties of float versus real
                self.chroma != cur_chroma
            } else if new_chroma > cur_chroma {
                let rgbe = viable_angle.rgb_with_chroma_and_value(new_chroma, cur_value);
                if let Some(new_rgb) = rgbe {
                    self.set_rgb(new_rgb);
                    // NB: beware frailties of float versus real
                    self.chroma != cur_chroma
                } else {
                    panic!("File: {:?} Line: {:?}", file!(), line!());
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn decr_value(&mut self, delta: f64) -> bool {
        assert!(is_proportion!(delta));
        let cur_value = self.rgb.value();
        if let Some(angle) = self.angle {
            let (min_value, _) = angle.value_range_with_chroma(self.chroma);
            let adj_delta = delta.min(cur_value - min_value);
            if adj_delta > 0.0 {
                let new_value = cur_value - adj_delta;
                let new_rgb = angle
                    .rgb_with_chroma_and_value(self.chroma, new_value)
                    .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()));
                self.set_rgb(new_rgb);
                // NB: beware frailties of float versus real
                new_rgb.value() != cur_value
            } else {
                false
            }
        } else {
            //RGB is grey
            if cur_value > 0.0 {
                let new_rgb = WHITE * (cur_value - delta).max(0.0);
                self.set_rgb(new_rgb);
                // NB: beware frailties of float versus real
                new_rgb.value() != cur_value
            } else {
                false
            }
        }
    }

    pub fn incr_value(&mut self, delta: f64) -> bool {
        assert!(is_proportion!(delta));
        let cur_value = self.rgb.value();
        if let Some(angle) = self.angle {
            let (_, max_value) = angle.value_range_with_chroma(self.chroma);
            let adj_delta = delta.min((max_value - cur_value).max(0.0));
            if adj_delta > 0.0 {
                let new_value = cur_value + adj_delta;
                let new_rgb = angle
                    .rgb_with_chroma_and_value(self.chroma, new_value)
                    .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()));
                self.set_rgb(new_rgb);
                // NB: beware frailties of float versus real
                new_rgb.value() != cur_value
            } else {
                false
            }
        } else {
            //RGB is grey
            if cur_value < 1.0 {
                let new_rgb = WHITE * (cur_value + delta).min(1.0);
                self.set_rgb(new_rgb);
                // NB: beware frailties of float versus real
                new_rgb.value() != cur_value
            } else {
                false
            }
        }
    }

    pub fn rotate(&mut self, by_angle: Angle) -> bool {
        if let Some(angle) = self.angle {
            let cur_value = self.rgb.value();
            let cur_chroma = self.chroma;
            let new_angle = angle + by_angle;
            let (min_value, max_value) = new_angle.value_range_with_chroma(cur_chroma);
            let new_rgb = if cur_value < min_value {
                new_angle
                    .rgb_with_chroma_and_value(cur_chroma, min_value)
                    .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()))
            } else if cur_value > max_value {
                new_angle
                    .rgb_with_chroma_and_value(cur_chroma, max_value)
                    .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()))
            } else {
                new_angle
                    .rgb_with_chroma_and_value(cur_chroma, cur_value)
                    .unwrap_or_else(|| panic!("File: {:?} Line: {:?}", file!(), line!()))
            };
            self.set_rgb(new_rgb);
            true
        } else {
            false
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

    #[test]
    fn rgb_math_rgb_manipulator_value() {
        let mut rgb_manipulator = RGBManipulator::new();
        assert_eq!(rgb_manipulator.rgb, WHITE);
        assert_eq!(rgb_manipulator.rgb.value(), 1.0);
        assert!(rgb_manipulator.angle.is_none());
        assert!(!rgb_manipulator.incr_value(0.1));
        let mut value = 1.0;
        loop {
            assert!(within_limit(rgb_manipulator.rgb.value(), value));
            assert!(rgb_manipulator.angle.is_none());
            value = (value - 0.1).max(0.0);
            if !rgb_manipulator.decr_value(0.1) {
                break;
            }
        }
        assert!(!rgb_manipulator.decr_value(0.1));
        value = 0.0;
        loop {
            assert!(within_limit(rgb_manipulator.rgb.value(), value));
            assert!(rgb_manipulator.angle.is_none());
            value = (value + 0.1).min(1.0);
            if !rgb_manipulator.incr_value(0.1) {
                break;
            }
        }
        assert!(!rgb_manipulator.incr_value(0.1));
        for rgb in [RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, (RED + YELLOW) / 2].iter() {
            rgb_manipulator.set_rgb(*rgb);
            assert!(!rgb_manipulator.incr_value(0.1));
            assert!(!rgb_manipulator.decr_value(0.1));
            let tint = (*rgb + WHITE) / 2.0;
            rgb_manipulator.set_rgb(tint);
            let angle = rgb_manipulator.angle.unwrap().angle();
            let chroma = rgb_manipulator.chroma;
            value = tint.value();
            while rgb_manipulator.decr_value(0.1) {
                assert!(rgb_manipulator.rgb.value() < value);
                assert!(!rgb_manipulator.angle.is_none());
                assert!(
                    (rgb_manipulator.angle.unwrap().angle() - angle).abs()
                        < Angle::from(0.00000001)
                );
                assert!(within_limit(rgb_manipulator.chroma, chroma));
                value = rgb_manipulator.rgb.value();
            }
            assert!(!rgb_manipulator.decr_value(0.1));
            value = rgb_manipulator.rgb.value();
            while rgb_manipulator.incr_value(0.1) {
                assert!(rgb_manipulator.rgb.value() > value);
                assert!(rgb_manipulator.angle.is_some());
                assert!(
                    (rgb_manipulator.angle.unwrap().angle() - angle).abs()
                        < Angle::from(0.00000001)
                );
                assert!(within_limit(rgb_manipulator.chroma, chroma));
                value = rgb_manipulator.rgb.value();
            }
            assert!(!rgb_manipulator.incr_value(0.1));
        }
    }

    #[test]
    fn rgb_math_rgb_manipulator_chroma() {
        let mut rgb_manipulator = RGBManipulator::new();
        assert!(!rgb_manipulator.incr_value(0.1));
        for rgb in [RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, (RED + YELLOW) / 2].iter() {
            let tint = (*rgb + WHITE) / 2.0;
            rgb_manipulator.set_rgb(tint);
            let angle = tint.angle().unwrap();
            assert!(
                (rgb_manipulator.angle.unwrap().angle() - angle).abs() < Angle::from(0.00000001)
            );
            let value = tint.value();
            let mut chroma = rgb_manipulator.chroma;
            while rgb_manipulator.decr_chroma(0.1) {
                assert!(rgb_manipulator.chroma < chroma);
                assert!(within_limit(rgb_manipulator.rgb.value(), value));
                if rgb_manipulator.angle.is_none() {
                    // last one will be grey
                    assert_eq!(rgb_manipulator.chroma, 0.0);
                } else {
                    assert!(
                        (rgb_manipulator.angle.unwrap().angle() - angle).abs()
                            < Angle::from(0.00000001)
                    );
                }
                chroma = rgb_manipulator.chroma;
            }
            assert!(rgb_manipulator.angle.is_none());
            assert!(!rgb_manipulator.decr_chroma(0.1));
            assert!(
                (rgb_manipulator.last_angle.unwrap().angle() - angle).abs()
                    < Angle::from(0.00000001)
            );
            while rgb_manipulator.incr_chroma(0.01) {
                assert!(rgb_manipulator.chroma > chroma);
                assert!(within_limit(rgb_manipulator.rgb.value(), value));
                assert!(!rgb_manipulator.angle.is_none());
                assert!(
                    (rgb_manipulator.angle.unwrap().angle() - angle).abs()
                        < Angle::from(0.00000001)
                );
                chroma = rgb_manipulator.chroma;
            }
        }
    }

    #[test]
    fn rgb_math_rgb_manipulator_rotate() {
        let mut rgb_manipulator = RGBManipulator::new();
        assert!(!rgb_manipulator.rotate(Angle::from(10.0)));
        assert!(!rgb_manipulator.rotate(-Angle::from(10.0)));
        for rgb in [RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, (RED + YELLOW) / 2].iter() {
            let tint = (*rgb + WHITE) / 2.0;
            rgb_manipulator.set_rgb(tint);
            for delta in [-60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0].iter() {
                let cur_chroma = rgb_manipulator.chroma;
                let cur_angle = rgb_manipulator.angle.unwrap();
                let delta_angle = Angle::from(*delta);
                rgb_manipulator.rotate(delta_angle);
                assert!(within_limit(cur_chroma, rgb_manipulator.chroma));
                let diff = rgb_manipulator.angle.unwrap() - cur_angle;
                assert!((diff - delta_angle).abs().radians() < 0.00000001);
            }
        }
        for rgb in [RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, (RED + YELLOW) / 2].iter() {
            let shade = *rgb * 0.5;
            rgb_manipulator.set_rgb(shade);
            for delta in [-60.0, -30.0, -10.0, -5.0, 5.0, 10.0, 30.0, 60.0].iter() {
                let cur_chroma = rgb_manipulator.chroma;
                let cur_angle = rgb_manipulator.angle.unwrap();
                let delta_angle = Angle::from(*delta);
                rgb_manipulator.rotate(delta_angle);
                assert!(within_limit(cur_chroma, rgb_manipulator.chroma));
                let diff = rgb_manipulator.angle.unwrap() - cur_angle;
                assert!((diff - delta_angle).abs().radians() < 0.00000001);
            }
        }
    }
}
