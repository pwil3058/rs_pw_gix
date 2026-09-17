// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::num::ParseIntError;

use crate::gdk;

pub fn format_geometry(event: &gdk::EventConfigure) -> String {
    let (x, y) = event.position();
    let (w, h) = event.size();
    format!("{}x{}+{}+{}", w, h, x, y)
}

pub fn format_geometry_size(event: &gdk::EventConfigure) -> String {
    let (w, h) = event.size();
    format!("{}x{}", w, h)
}

pub fn format_geometry_position(event: &gdk::EventConfigure) -> String {
    let (x, y) = event.position();
    format!("{}+{}", x, y)
}

pub fn parse_geometry(text: &str) -> Result<(i32, i32, i32, i32), ParseIntError> {
    let v: Vec<&str> = text.splitn(2, '+').collect();
    let (width, height) = parse_geometry_size(v[0])?;
    let (x, y) = parse_geometry_position(v[1])?;
    Ok((width, height, x, y))
}

pub fn parse_geometry_size(text: &str) -> Result<(i32, i32), ParseIntError> {
    let v: Vec<&str> = text.splitn(2, 'x').collect();
    let width = v[0].parse::<i32>()?;
    let height = v[1].parse::<i32>()?;
    Ok((width, height))
}

pub fn parse_geometry_position(text: &str) -> Result<(i32, i32), ParseIntError> {
    let v: Vec<&str> = text.splitn(2, '+').collect();
    let x = v[0].parse::<i32>()?;
    let y = v[1].parse::<i32>()?;
    Ok((x, y))
}
