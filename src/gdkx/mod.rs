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

use std::num::ParseIntError;

use gdk;

pub fn format_geometry(event: &gdk::EventConfigure) -> String {
    let (x, y) = event.get_position();
    let (w, h) = event.get_size();
    format!("{}x{}+{}+{}", w, h, x, y)
}

pub fn format_geometry_size(event: &gdk::EventConfigure) -> String {
    let (w, h) = event.get_size();
    format!("{}x{}", w, h)
}

pub fn format_geometry_position(event: &gdk::EventConfigure) -> String {
    let (x, y) = event.get_position();
    format!("{}+{}", x, y)
}

pub fn parse_geometry(text: &str) -> Result<(i32, i32, i32, i32), ParseIntError> {
    let v: Vec<&str> = text.splitn(2, "+").collect();
    let (width, height) = parse_geometry_size(v[0])?;
    let (x, y) = parse_geometry_position(v[1])?;
    Ok((width, height, x, y))
}

pub fn parse_geometry_size(text: &str) -> Result<(i32, i32), ParseIntError> {
    let v: Vec<&str> = text.splitn(2, "x").collect();
    let width = v[0].parse::<i32>()?;
    let height = v[1].parse::<i32>()?;
    Ok((width, height))
}

pub fn parse_geometry_position(text: &str) -> Result<(i32, i32), ParseIntError> {
    let v: Vec<&str> = text.splitn(2, "+").collect();
    let x = v[0].parse::<i32>()?;
    let y = v[1].parse::<i32>()?;
    Ok((x, y))
}
