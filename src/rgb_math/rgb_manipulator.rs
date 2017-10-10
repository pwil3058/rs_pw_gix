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

use std::cell::{RefCell, RefMut};

use ::rgb_math::hue::*;
use ::rgb_math::rgb::*;

struct RGBManipulator {
    rgb: RefCell<RGB>,
    angle: RefCell<HueAngle>,
    last_angle: RefCell<HueAngle>,
    chroma: RefCell<f64>
}

impl RGBManipulator {
    pub fn new() -> RGBManipulator {
        let rgb = RefCell::new(WHITE);
        let angle = RefCell::new(HueAngle::from(*rgb.borrow()));
        let last_angle = RefCell::new(HueAngle::from(RED));
        let chroma = RefCell::new(rgb.borrow().hypot() * angle.borrow().get_chroma_correction());
        RGBManipulator{rgb, angle, last_angle, chroma}
    }

    pub fn set_rgb(&self, rgb:RGB) {
        *self.rgb.borrow_mut() = rgb;
        let new_angle = HueAngle::from(rgb);
        *self.chroma.borrow_mut() = rgb.hypot() * new_angle.get_chroma_correction();
        let is_grey = new_angle.is_grey();
        if !is_grey {
            *self.last_angle.borrow_mut() = new_angle;
        };
        *self.angle.borrow_mut() = new_angle;
    }

    pub fn get_rgb(&self) -> RGB {
        *self.rgb.borrow()
    }

    pub fn decr_chroma(&self, delta: f64) -> bool {
        assert!(is_proportion!(delta));
        let is_grey = self.angle.borrow().is_grey();
        if is_grey {
            false
        } else {
            let cur_value = self.rgb.borrow().value();
            let new_chroma = (*self.chroma.borrow() - delta).max(0.0);
            let new_rgb = self.angle.borrow().rgb_with_chroma_and_value(new_chroma, cur_value).unwrap_or_else(
                || panic!("File: {:?} Line: {:?}", file!(), line!())
            );
            self.set_rgb(new_rgb);
            true
       }
    }

    pub fn incr_chroma(&self, delta: f64) -> bool {
        assert!(is_proportion!(delta));
        let cur_value = self.rgb.borrow().value();
        let viable_angle = if self.angle.borrow().is_grey() {
            self.last_angle.borrow()
        } else {
            self.angle.borrow()
        };
        let max_chroma = viable_angle.max_chroma_for_value(cur_value);
        let adj_delta = delta.min(max_chroma - *self.chroma.borrow());
        if adj_delta > 0.0 {
            let new_chroma = *self.chroma.borrow() + adj_delta;
            let new_rgb = viable_angle.rgb_with_chroma_and_value(new_chroma, cur_value).unwrap_or_else(
                || panic!("File: {:?} Line: {:?}", file!(), line!())
            );
            self.set_rgb(new_rgb);
            true
        } else {
            false
        }
    }

    pub fn decr_value(&self, delta: f64) -> bool {
        assert!(is_proportion!(delta));
        let cur_value = self.rgb.borrow().value();
        let value_range = self.angle.borrow().value_range_with_chroma(*self.chroma.borrow());
        match value_range {
            Some((min_value, _)) => {
                let adj_delta = delta.min(cur_value - min_value);
                if adj_delta > 0.0 {
                    let new_value = cur_value - adj_delta;
                    let new_rgb = self.angle.borrow().rgb_with_chroma_and_value(*self.chroma.borrow(), new_value).unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    self.set_rgb(new_rgb);
                    true
                } else {
                    false
                }
            },
            None => { //RGB is grey
                if cur_value > 0.0 {
                    let new_value = (cur_value - delta).max(0.0);
                    self.set_rgb(WHITE * new_value);
                    true
                } else {
                    false
                }

            }
        }
    }

    pub fn incr_value(&self, delta: f64) -> bool {
        assert!(is_proportion!(delta));
        let cur_value = self.rgb.borrow().value();
        let value_range = self.angle.borrow().value_range_with_chroma(*self.chroma.borrow());
        match value_range {
            Some((_, max_value)) => {
                let adj_delta = delta.min((max_value - cur_value).max(0.0));
                if adj_delta > 0.0 {
                    let new_value = cur_value + adj_delta;
                    let new_rgb = self.angle.borrow().rgb_with_chroma_and_value(*self.chroma.borrow(), new_value).unwrap_or_else(
                        || panic!("File: {:?} Line: {:?}", file!(), line!())
                    );
                    self.set_rgb(new_rgb);
                    true
                } else {
                    false
                }
            },
            None => { //RGB is grey
                if cur_value < 1.0 {
                    let new_value = (cur_value + delta).min(1.0);
                    self.set_rgb(WHITE * new_value);
                    true
                } else {
                    false
                }

            }
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

    #[test]
    fn rgb_math_rgb_manipulator_value() {
        let rgb_manipulator = RGBManipulator::new();
        assert_eq!(*rgb_manipulator.rgb.borrow(), WHITE);
        assert_eq!(rgb_manipulator.rgb.borrow().value(), 1.0);
        assert!(rgb_manipulator.angle.borrow().is_grey());
        assert!(!rgb_manipulator.incr_value(0.1));
        let mut value = 1.0;
        loop {
            assert!(within_limit(rgb_manipulator.rgb.borrow().value(), value));
            assert!(rgb_manipulator.angle.borrow().is_grey());
            value = (value - 0.1).max(0.0);
            if !rgb_manipulator.decr_value(0.1) {
                break;
            }
        };
        assert!(!rgb_manipulator.decr_value(0.1));
        value = 0.0;
        loop {
            assert!(within_limit(rgb_manipulator.rgb.borrow().value(), value));
            assert!(rgb_manipulator.angle.borrow().is_grey());
            value = (value + 0.1).min(1.0);
            if !rgb_manipulator.incr_value(0.1) {
                break;
            }
        };
        assert!(!rgb_manipulator.incr_value(0.1));
        rgb_manipulator.set_rgb(RED);
        assert!(!rgb_manipulator.incr_value(0.1));
        assert!(!rgb_manipulator.decr_value(0.1));
        rgb_manipulator.set_rgb((RED + WHITE) / 2.0);
        let angle = rgb_manipulator.angle.borrow().get_angle();
        value = 2.0 / 3.0;
        while rgb_manipulator.decr_value(0.1) {
            assert!(rgb_manipulator.rgb.borrow().value() < value);
            assert!(!rgb_manipulator.angle.borrow().is_grey());
            assert_eq!(rgb_manipulator.angle.borrow().get_angle(), angle);
            value = rgb_manipulator.rgb.borrow().value();
        };
        assert!(!rgb_manipulator.decr_value(0.1));
        value = rgb_manipulator.rgb.borrow().value();
        while rgb_manipulator.incr_value(0.1) {
            assert!(rgb_manipulator.rgb.borrow().value() > value);
            assert!(!rgb_manipulator.angle.borrow().is_grey());
            assert_eq!(rgb_manipulator.angle.borrow().get_angle(), angle);
            value = rgb_manipulator.rgb.borrow().value();
        };
        assert!(!rgb_manipulator.incr_value(0.1));
    }
}
