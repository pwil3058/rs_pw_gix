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

use std::cmp::{PartialEq, Eq, PartialOrd, Ordering};
use std::convert::From;
use std::rc::Rc;

pub mod attributes;

use ::rgb_math::hue::*;
use ::rgb_math::rgb::*;

pub trait ColourInterface {
    fn rgb(&self) -> RGB;
    fn hue(&self) -> HueAngle;
    fn is_grey(&self) -> bool;
    fn chroma(&self) -> f64;
    fn greyness(&self) -> f64;
    fn value(&self) -> f64 ;
    fn warmth(&self) -> f64 ;
    fn monotone_rgb(&self) -> RGB;
    fn best_foreground_rgb(&self) -> RGB;
    fn max_chroma_rgb(&self) -> RGB;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ColourInternals {
    rgb: RGB,
    hue: HueAngle
}

impl PartialEq for ColourInternals {
    fn eq(&self, other: &ColourInternals) -> bool {
        self.rgb == other.rgb
    }
}

impl Eq for ColourInternals {}

impl PartialOrd for ColourInternals {
    fn partial_cmp(&self, other: &ColourInternals) -> Option<Ordering> {
        if self.rgb == other.rgb {
            Some(Ordering::Equal)
        } else if self.hue.is_grey() {
            if other.hue.is_grey() {
                self.rgb.value().partial_cmp(&other.rgb.value())
            } else {
                Some(Ordering::Less)
            }
        } else if other.hue.is_grey() {
            Some(Ordering::Greater)
        } else {
            self.hue.angle().radians().partial_cmp(&other.hue.angle().radians())
        }
    }
}

pub type Colour = Rc<ColourInternals>;

impl From<RGB> for Colour {
    fn from(rgb: RGB) -> Colour {
        let hue = HueAngle::from(rgb);
        Rc::new(ColourInternals{rgb, hue})
    }
}

impl ColourInterface for Colour {
    fn rgb(&self) -> RGB {
        self.rgb
    }

    fn hue(&self) -> HueAngle {
        self.hue
    }

    fn is_grey(&self) -> bool {
        self.hue.is_grey()
    }

    fn chroma(&self) -> f64 {
        self.rgb.hypot() * self.hue.chroma_correction()
    }

    fn greyness(&self) -> f64 {
        1.0 - self.rgb.hypot() * self.hue.chroma_correction()
    }

    fn value(&self) -> f64 {
        self.rgb.value()
    }

    fn warmth(&self) -> f64 {
        (self.rgb.x() + 1.0) / 2.0
    }

    fn monotone_rgb(&self) -> RGB {
        WHITE * self.rgb.value()
    }

    fn best_foreground_rgb(&self) -> RGB {
        self.rgb().best_foreground_rgb()
    }

    fn max_chroma_rgb(&self) -> RGB {
        self.hue.max_chroma_rgb()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::rgb_math::angle::*;

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
    fn colour_basics() {
        for rgb in [RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, (RED + YELLOW) / 2].iter() {
            let shade = *rgb * 0.5;
            let colour = Colour::from(shade);
            assert_eq!(colour.max_chroma_rgb(), *rgb);
            assert_eq!(colour.rgb(), shade);
            assert_eq!(colour.value(), shade.value());
            assert_eq!(colour.chroma(), 0.5);
        }
        for rgb in [RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, (RED + YELLOW) / 2].iter() {
            let tint = (*rgb + WHITE) * 0.5;
            let colour = Colour::from(tint);
            assert_eq!(colour.max_chroma_rgb(), *rgb);
            assert_eq!(colour.rgb(), tint);
            assert_eq!(colour.value(), tint.value());
            assert_eq!(colour.chroma(), 0.5);
        }
        for factor in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
            assert_eq!(Colour::from(RED * *factor).warmth(), (1.0 * factor + 1.0) / 2.0);
            assert_eq!(Colour::from(((RED + WHITE) * 0.5) * *factor).warmth(), (0.5 * factor + 1.0) / 2.0);
        }
        for factor in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
            assert_eq!(Colour::from(CYAN * *factor).warmth(), (-1.0 * factor + 1.0) / 2.0);
            assert_eq!(Colour::from(((CYAN + WHITE) * 0.5) * *factor).warmth(), (-0.5 * factor + 1.0) / 2.0);
        }
        for rgb in [YELLOW, MAGENTA].iter() {
            for factor in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                let tint = (*rgb + WHITE) /2.0;
                assert!(within_limit(Colour::from(*rgb * *factor).warmth(), (factor * DEG_60.cos() + 1.0) / 2.0));
                assert!(within_limit(Colour::from(tint * *factor).warmth(), (0.5 * factor * DEG_60.cos() + 1.0) / 2.0));
            }
        }
        for rgb in [GREEN, BLUE].iter() {
            for factor in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter() {
                let tint = (*rgb + WHITE) /2.0;
                assert!(within_limit(Colour::from(*rgb * *factor).warmth(), (factor * DEG_120.cos() + 1.0) / 2.0));
                assert!(within_limit(Colour::from(tint * *factor).warmth(), (0.5 * factor * DEG_120.cos() + 1.0) / 2.0));
            }
        }
    }
}