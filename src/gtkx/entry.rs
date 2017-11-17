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

use std::cell::{Cell, RefCell};
use std::cmp;
use std::rc::Rc;

use gdk;
use glib::signal::Inhibit;
use gtk;
use gtk::prelude::*;

use gtkx::coloured::*;
use rgb_math::rgb::*;

pub trait HexEntryInterface {
    type HexEntryType;
    type PackableWidgetType;

    fn create() -> Self::HexEntryType;
    fn create_with_max(max_value: u32) -> Self::HexEntryType;
    fn pwo(&self) -> Self::PackableWidgetType;

    fn get_value(&self) -> u32;
    fn set_value(&self, value: u32);
    fn set_value_from_str(&self, text: &str);
    fn incr_value(&self, incr: u32) -> bool;
    fn decr_value(&self, decr: u32) -> bool;

    fn connect_value_changed<F: 'static + Fn(u32)>(&self, callback: F);
    fn inform_value_changed(&self);
}

//#[derive(Debug)]
pub struct HexEntryData {
    entry: gtk::Entry,
    value: Cell<u32>,
    max_value: u32,
    current_step: Cell<u32>,
    max_step: u32,
    width: usize,
    callbacks: RefCell<Vec<Box<Fn(u32)>>>
}

impl HexEntryData {
    fn bump_current_step(&self) {
        let new_step = cmp::min(self.current_step.get() + 1, self.max_step);
        self.current_step.set(new_step);
    }

    fn reset_current_step(&self) {
        self.current_step.set(1);
    }

    fn reset_entry_text(&self) {
        self.entry.set_text(&format!("0x{:0width$X}", self.value.get(), width = self.width));
    }
}

type HexEntry = Rc<HexEntryData>;

fn sig_hex_digits(mut max_value: u32) -> usize {
    let mut width:usize = 0;
    while max_value != 0 {
        width += 1;
        max_value /= 16
    }
    width
}

impl HexEntryInterface for HexEntry {
    type HexEntryType = HexEntry;
    type PackableWidgetType = gtk::Entry;

    fn create() -> Self::HexEntryType {
        Self::create_with_max(u32::max_value())
    }

    fn create_with_max(max_value: u32) -> HexEntry {
        let entry = gtk::Entry::new();
        let value: Cell<u32> = Cell::new(0);
        let width = sig_hex_digits(max_value);
        let max_step = cmp::max(max_value / 32, 1);
        let current_step: Cell<u32> = Cell::new(1);
        let callbacks: RefCell<Vec<Box<Fn(u32)>>> = RefCell::new(Vec::new());
        entry.set_width_chars(width as i32 + 2);
        entry.set_text(&format!("0x{:0width$X}", value.get(), width = width));
        let hex_entry = Rc::new(HexEntryData{entry, value, max_value, width, current_step, max_step, callbacks});
        let hec = hex_entry.clone();
        hex_entry.entry.connect_key_press_event(
            move |widget, event_key| {
                let key = event_key.get_keyval();
                match key {
                    gdk::enums::key::Return | gdk::enums::key::Tab => {
                        if let Some(text) = widget.get_text() {
                            hec.set_value_from_str(&text);
                        } else {
                            hec.reset_entry_text();
                        }
                        // NB: this will nobble the "activate" signal
                        // but let the Tab key move the focus
                        Inhibit(key == gdk::enums::key::Return)
                    },
                    gdk::enums::key::Up => {
                        if hec.incr_value(hec.current_step.get()) {
                            hec.bump_current_step()
                        }
                        Inhibit(true)
                    },
                    gdk::enums::key::Down => {
                        if hec.decr_value(hec.current_step.get()) {
                            hec.bump_current_step()
                        }
                        Inhibit(true)
                    },
                    _ => Inhibit(false)
                }
            }
        );
        let hec = hex_entry.clone();
        hex_entry.entry.connect_key_release_event(
            move |_, event_key| {
                if [gdk::enums::key::Up, gdk::enums::key::Down].contains(&event_key.get_keyval()) {
                    hec.reset_current_step();
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            }
        );
        hex_entry
    }

    fn pwo(&self) -> gtk::Entry {
        self.entry.clone()
    }

    fn get_value(&self) -> u32 {
        self.value.get()
    }

    fn set_value(&self, value: u32) {
        if value <= self.max_value {
            self.value.set(value);
            self.reset_entry_text();
            self.inform_value_changed()
        } else {
            // TODO: think about panicking here
        }
    }

    fn incr_value(&self, incr: u32) -> bool {
        let value = self.value.get();
        let adj_incr = cmp::min(self.max_value - value, incr);
        if adj_incr > 0 {
            self.set_value(self.value.get() + adj_incr);
        }
        self.value.get() < self.max_value
    }

    fn decr_value(&self, decr: u32) -> bool {
        let value = self.value.get();
        let adj_decr = cmp::min(value, decr);
        if decr > 0 {
            self.set_value(value - adj_decr);
        }
        self.value.get() > 0
    }

    fn set_value_from_str(&self, text: &str) {
        let value_e = if let Some(index) = text.find("x") {
            u32::from_str_radix(&text[index + 1..], 16)
        } else {
            u32::from_str_radix(text, 16)
        };
        if let Ok(value) = value_e {
            self.set_value(value);
        } else {
            self.reset_entry_text();
        }
    }

    fn connect_value_changed<F: 'static + Fn(u32)>(&self, callback: F) {
        self.callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_value_changed(&self) {
        let value = self.value.get();
        for callback in self.callbacks.borrow().iter() {
            callback(value);
        }
    }
}


pub trait RGBEntryInterface {
    type RGBEntryType;
    type PackableWidgetType;

    fn create() -> Self::RGBEntryType;
    fn pwo(&self) -> Self::PackableWidgetType;

    fn get_rgb(&self) -> RGB;
    fn set_rgb(&self, rgb: &RGB);

    fn connect_value_changed<F: 'static + Fn(&RGB)>(&self, callback: F);
    fn inform_value_changed(&self);
}

pub struct RGBHexEntryBoxData {
    hbox: gtk::Box,
    red_entry: HexEntry,
    green_entry: HexEntry,
    blue_entry: HexEntry,
    callbacks: RefCell<Vec<Box<Fn(&RGB)>>>
}

pub type RGBHexEntryBox = Rc<RGBHexEntryBoxData>;

impl RGBEntryInterface for RGBHexEntryBox {
    type RGBEntryType = RGBHexEntryBox;
    type PackableWidgetType = gtk::Box;

    fn create() -> RGBHexEntryBox {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
        let max_value = u16::max_value() as u32;
        let red_label = gtk::Label::new("Red");
        red_label.set_widget_colour_rgb(&RED);
        let red_entry = HexEntry::create_with_max(max_value);
        let green_label = gtk::Label::new("Green");
        green_label.set_widget_colour_rgb(&GREEN);
        let green_entry = HexEntry::create_with_max(max_value);
        let blue_label = gtk::Label::new("Blue");
        blue_label.set_widget_colour_rgb(&BLUE);
        let blue_entry = HexEntry::create_with_max(max_value);
        hbox.pack_start(&red_label, true, true, 0);
        hbox.pack_start(&red_entry.pwo(), true, true, 0);
        hbox.pack_start(&green_label, true, true, 0);
        hbox.pack_start(&green_entry.pwo(), true, true, 0);
        hbox.pack_start(&blue_label, true, true, 0);
        hbox.pack_start(&blue_entry.pwo(), true, true, 0);
        let callbacks: RefCell<Vec<Box<Fn(&RGB)>>> = RefCell::new(Vec::new());
        let rgb_entry_box = Rc::new(RGBHexEntryBoxData{hbox, red_entry, green_entry, blue_entry, callbacks});
        let reb = rgb_entry_box.clone();
        rgb_entry_box.red_entry.connect_value_changed(
            move |_| {reb.inform_value_changed();}
        );
        let reb = rgb_entry_box.clone();
        rgb_entry_box.green_entry.connect_value_changed(
            move |_| {reb.inform_value_changed();}
        );
        let reb = rgb_entry_box.clone();
        rgb_entry_box.blue_entry.connect_value_changed(
            move |_| {reb.inform_value_changed();}
        );
        rgb_entry_box
    }

    fn pwo(&self) -> gtk::Box {
        self.hbox.clone()
    }

    fn get_rgb(&self) -> RGB {
        let max_value = u16::max_value() as f64;
        let red = self.red_entry.get_value() as f64 / max_value;
        let green = self.green_entry.get_value() as f64 / max_value;
        let blue = self.blue_entry.get_value() as f64 / max_value;
        RGB::from((red, green, blue))
    }

    fn set_rgb(&self, rgb: &RGB) {
        let max_value = u16::max_value() as f64;
        let red = (rgb.red * max_value) as u32;
        let green = (rgb.green * max_value) as u32;
        let blue = (rgb.blue * max_value) as u32;
        self.red_entry.set_value(red);
        self.green_entry.set_value(green);
        self.blue_entry.set_value(blue);
    }

    fn connect_value_changed<F: 'static + Fn(&RGB)>(&self, callback: F) {
        self.callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_value_changed(&self) {
        let rgb = self.get_rgb();
        for callback in self.callbacks.borrow().iter() {
            callback(&rgb);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtkx_entry_rgb_entry_box() {
        if !gtk::is_initialized() {
            if let Err(err) = gtk::init() {
                panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
            };
        }

        let rgb_entry_box = RGBHexEntryBox::create();
        let rgb = rgb_entry_box.get_rgb();
        println!("{:?} {:?}", rgb, BLACK);
        assert_eq!(rgb, BLACK);
    }
}
