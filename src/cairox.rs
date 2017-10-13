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

use std::f64::consts;

use cairo;

use colour::Colour;
use rgb_math::rgb::RGB;

pub type Point = (f64, f64);
pub type Points = Vec<Point>;

pub enum Side {
    Top,
    Bottom,
    Left,
    Right
}

pub trait Draw {
    fn draw_circle(&self, centre: Point, radius: f64, filled: bool);
    fn draw_line(&self, start: Point, end: Point);
    fn draw_polygon(&self, polygon: Points, fill: bool);
    fn draw_indicator(&self, position: Point, side: Side, size: f64);
    fn set_source_colour(&self, rgb: Colour);
    fn set_source_colour_rgb(&self, rgb: RGB);
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

    fn set_source_colour(&self, colour: Colour) {
        self.set_source_colour_rgb(colour.rgb())
    }

    fn set_source_colour_rgb(&self, rgb: RGB) {
        self.set_source_rgb(rgb[0], rgb[1], rgb[2])
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
