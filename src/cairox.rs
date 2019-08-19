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
use gdk::prelude::ContextExt;
use gdk_pixbuf::Pixbuf;

use crate::colour::*;
use crate::rgb_math::rgb::RGB;

pub use crate::geometry::*;

/// Direction in which to draw indicators
pub enum Dirn {
    Down,
    Up,
    Right,
    Left,
}

pub trait Draw {
    fn draw_circle(&self, centre: Point, radius: f64, fill: bool);
    fn draw_diamond(&self, centre: Point, side_length: f64, filled: bool);
    fn draw_line(&self, start: Point, end: Point);
    fn draw_polygon(&self, polygon: Points, fill: bool);
    fn draw_square(&self, centre: Point, side_length: f64, filled: bool);
    fn draw_indicator(&self, position: Point, dirn: Dirn, size: f64);
    fn move_to_point(&self, point: Point);
    fn line_to_point(&self, point: Point);
    fn set_source_colour(&self, rgb: &Colour);
    fn set_source_colour_rgb(&self, rgb: RGB);
    fn set_source_surface_at(&self, surface: &cairo::Surface, position: Point);
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

    fn draw_diamond(&self, centre: Point, side_length: f64, fill: bool) {
        let dist = side_length * COS_45_DEG;
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

    fn draw_square(&self, centre: Point, side_length: f64, fill: bool) {
        let start_x = centre.0 - side_length / 2.0;
        let start_y = centre.1 - side_length / 2.0;
        self.move_to(start_x, start_y);
        self.line_to(start_x + side_length, start_y);
        self.line_to(start_x + side_length, start_y + side_length);
        self.line_to(start_x, start_y + side_length);
        self.close_path();
        if fill {
            self.fill();
        } else {
            self.stroke();
        }
    }

    fn draw_indicator(&self, position: Point, dirn: Dirn, size: f64) {
        self.move_to(position.0, position.1);
        match dirn {
            Dirn::Down => {
                self.line_to(position.0 + size / 2.0, position.1);
                self.line_to(position.0, position.1 + size);
                self.line_to(position.0 - size / 2.0, position.1);
            }
            Dirn::Up => {
                self.line_to(position.0 + size / 2.0, position.1);
                self.line_to(position.0, position.1 - size);
                self.line_to(position.0 - size / 2.0, position.1);
            }
            Dirn::Right => {
                self.line_to(position.0, position.1 + size / 2.0);
                self.line_to(position.0 + size, position.1);
                self.line_to(position.0, position.1 - size / 2.0);
            }
            Dirn::Left => {
                self.line_to(position.0, position.1 + size / 2.0);
                self.line_to(position.0 - size, position.1);
                self.line_to(position.0, position.1 - size / 2.0);
            }
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

    fn set_source_surface_at(&self, surface: &cairo::Surface, position: Point) {
        self.set_source_surface(surface, position.0, position.1)
    }

    fn set_source_pixbuf_at(&self, pixbuf: &Pixbuf, position: Point, with_border: bool) {
        if !with_border {
            // TODO: find out how to kill border
            let for_window: Option<&gdk::Window> = None;
            if let Some(surface) = Self::cairo_surface_create_from_pixbuf(pixbuf, 0, for_window) {
                self.set_source_surface_at(&surface, position);
                return;
            };
        };
        self.set_source_pixbuf(pixbuf, position.0, position.1);
    }
}
