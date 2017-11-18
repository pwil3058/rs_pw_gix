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

use std::convert::From;
use std::f64::consts;
use std::ops::*;

use cairo;
use gdk;
use gdk::prelude::ContextExt;
use gdk_pixbuf::Pixbuf;

use colour::*;
use rgb_math::angle::Angle;
use rgb_math::rgb::RGB;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point (pub f64, pub f64);

const SQRT_2: f64 = 1.4142_13562_37309_50488;
const SIN_45_DEG: f64 = 1.0 / SQRT_2;
const COS_45_DEG: f64 = SIN_45_DEG;

impl Point {
    pub fn hypot(&self) -> f64 {
        self.0.hypot(self.1)
    }

    pub fn x(&self) -> f64 {
        self.0
    }

    pub fn y(&self) -> f64 {
        self.1
    }

    pub fn xy(&self) -> (f64, f64) {
        (self.0, self.1)
    }

    pub fn rotate_45_deg(&self) -> Point {
        Point(self.0 - self.1, self.0 + self.1) * SIN_45_DEG
    }
}

impl From<(f64, f64)> for Point {
    fn from(tuple: (f64,f64)) -> Point {
        Point(tuple.0, tuple.1)
    }
}

impl From<(Angle, f64)> for Point {
    fn from(polar: (Angle, f64)) -> Point {
        // NB: cairo coordinates are upside down to normal people
        let (angle, radius) = polar;
        if angle.is_nan() {
            Point(0.0, -radius)
        } else {
            Point(radius * angle.cos(), -radius * angle.sin())
        }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Point) {
        *self = *self + rhs
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Point {
        Point(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Point) {
        *self = *self - rhs
    }
}

impl<Scalar: Into<f64> + Copy> Mul<Scalar> for Point {
    type Output = Point;

    fn mul(self, rhs: Scalar) -> Point {
        let frhs: f64 = rhs.into();
        Point(self.0 * frhs, self.1 * frhs)
    }
}

impl<Scalar: Into<f64> + Copy> MulAssign<Scalar> for Point {
    fn mul_assign(&mut self, rhs: Scalar) {
        *self = *self * rhs
    }
}

impl<Scalar: Into<f64> + Copy> Div<Scalar> for Point {
    type Output = Point;

    fn div(self, rhs: Scalar) -> Point {
        let frhs: f64 = rhs.into();
        Point(self.0 / frhs, self.1 / frhs)
    }
}

impl<Scalar: Into<f64> + Copy> DivAssign<Scalar> for Point {
    fn div_assign(&mut self, rhs: Scalar) {
        *self = *self / rhs
    }
}

pub type Points = Vec<Point>;

pub enum Side {
    Top,
    Bottom,
    Left,
    Right
}

pub trait Draw {
    fn draw_circle(&self, centre: Point, radius: f64, fill: bool);
    fn draw_diamond(&self, centre: Point, side: f64, filled: bool);
    fn draw_line(&self, start: Point, end: Point);
    fn draw_polygon(&self, polygon: Points, fill: bool);
    fn draw_square(&self, centre: Point, side: f64, filled: bool);
    fn draw_indicator(&self, position: Point, side: Side, size: f64);
    fn move_to_point(&self, point: Point);
    fn line_to_point(&self, point: Point);
    fn set_source_colour(&self, rgb: &Colour);
    fn set_source_colour_rgb(&self, rgb: RGB);
    fn set_source_surface_at<T: AsRef<cairo::Surface>>(&self, surface: &T, position: Point);
    fn set_source_pixbuf_at(&self, pixbuf: &Pixbuf, position: Point, with_border: bool);
}

impl Draw for cairo::Context {
    fn draw_circle(&self, centre: Point, radius: f64, fill: bool) {
        self.arc(centre.0, centre.1, radius, 0.0, 2.0 * consts::PI);
        if fill {
            self.fill();
        } else {
            self.stroke();
        }
    }

    fn draw_diamond(&self, centre: Point, side: f64, fill: bool) {
        let dist = side * COS_45_DEG;
        self.move_to(centre.0, centre.1 + dist);
        self.line_to(centre.0 + dist, centre.1);
        self.line_to(centre.0, centre.1 - dist);
        self.line_to(centre.0 - dist, centre.1);
        self.close_path();
        if fill {
            self.fill();
        } else {
            self.stroke();
        }
    }

    fn draw_line(&self, start: Point, end: Point) {
        self.move_to(start.0, start.1);
        self.line_to(end.0, end.1);
        self.stroke();
    }

    fn draw_polygon(&self, polygon: Points, fill: bool) {
        self.move_to(polygon[0].0, polygon[0].1);
        for index in 1..polygon.len() {
            self.line_to(polygon[index].0, polygon[index].1);
        }
        self.close_path();
        if fill {
            self.fill();
        } else {
            self.stroke();
        }
    }

    fn draw_square(&self, centre: Point, side: f64, fill: bool) {
        let start_x = centre.0 - side / 2.0;
        let start_y = centre.1 - side / 2.0;
        self.move_to(start_x, start_y);
        self.line_to(start_x + side, start_y);
        self.line_to(start_x + side, start_y + side);
        self.line_to(start_x, start_y + side);
        self.close_path();
        if fill {
            self.fill();
        } else {
            self.stroke();
        }
    }

    fn draw_indicator(&self, position: Point, side: Side, size: f64) {
        self.move_to(position.0, position.1);
        match side {
            Side::Top => {
                self.line_to(position.0 + size / 2.0, position.1);
                self.line_to(position.0, position.1 + size);
                self.line_to(position.0 - size / 2.0, position.1);
            },
            Side::Bottom => {
                self.line_to(position.0 + size / 2.0, position.1);
                self.line_to(position.0, position.1 - size);
                self.line_to(position.0 - size / 2.0, position.1);
            },
            Side::Left => {
                self.line_to(position.0, position.1 + size / 2.0);
                self.line_to(position.0 + size, position.1);
                self.line_to(position.0, position.1 - size / 2.0);
            },
            Side::Right => {
                self.line_to(position.0, position.1 + size / 2.0);
                self.line_to(position.0 - size, position.1);
                self.line_to(position.0, position.1 - size / 2.0);
            },
        }
        self.close_path();
        self.fill();
    }

    fn move_to_point(&self, point: Point) {
        self.move_to(point.0, point.1);
    }

    fn line_to_point(&self, point: Point) {
        self.line_to(point.0, point.1);
    }

    fn set_source_colour(&self, colour: &Colour) {
        self.set_source_colour_rgb(colour.rgb())
    }

    fn set_source_colour_rgb(&self, rgb: RGB) {
        self.set_source_rgb(rgb[0], rgb[1], rgb[2])
    }

    fn set_source_surface_at<T: AsRef<cairo::Surface>>(&self, surface: &T, position: Point) {
        self.set_source_surface(surface, position.0, position.1)
    }

    fn set_source_pixbuf_at(&self, pixbuf: &Pixbuf, position: Point, with_border: bool) {
        if !with_border {
            // TODO: find out how to kill border
            if let Some(surface) = Self::cairo_surface_create_from_pixbuf(pixbuf, 0, None) {
                self.set_source_surface_at(&surface, position);
                return;
            };
        };
        self.set_source_pixbuf(pixbuf, position.0, position.1);
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
