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
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::rc::Rc;

use gdk;
use glib::signal::Inhibit;
use gtk;
use gtk::prelude::*;

use pw_pathux;

use gtkx::coloured::*;
use gtkx::list_store::*;
use rgb_math::rgb::*;
use wrapper::*;

pub trait HexEntryInterface {
    fn create() -> Self;
    fn create_with_max(max_value: u32) -> Self;
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

impl_widget_wrapper!(HexEntryData, entry, gtk::Entry);

impl HexEntryData {
    pub fn get_value(&self) -> u32 {
        self.value.get()
    }

    pub fn set_value(&self, value: u32) {
        if value <= self.max_value {
            self.value.set(value);
            self.reset_entry_text();
        } else {
            // TODO: think about panicking here
        }
    }

    pub fn incr_value(&self, incr: u32) -> bool {
        let value = self.value.get();
        let adj_incr = cmp::min(self.max_value - value, incr);
        if adj_incr > 0 {
            self.set_value_and_notify(self.value.get() + adj_incr);
        }
        self.value.get() < self.max_value
    }

    pub fn decr_value(&self, decr: u32) -> bool {
        let value = self.value.get();
        let adj_decr = cmp::min(value, decr);
        if decr > 0 {
            self.set_value_and_notify(value - adj_decr);
        }
        self.value.get() > 0
    }

    pub fn set_value_from_str(&self, text: &str) {
        let value_e = if let Some(index) = text.find("x") {
            u32::from_str_radix(&text[index + 1..], 16)
        } else {
            u32::from_str_radix(text, 16)
        };
        if let Ok(value) = value_e {
            self.set_value_and_notify(value);
        } else {
            self.reset_entry_text();
        }
    }

    pub fn connect_value_changed<F: 'static + Fn(u32)>(&self, callback: F) {
        self.callbacks.borrow_mut().push(Box::new(callback))
    }

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

    fn set_value_and_notify(&self, value: u32) {
        self.set_value(value);
        self.inform_value_changed();
    }

    fn inform_value_changed(&self) {
        let value = self.value.get();
        for callback in self.callbacks.borrow().iter() {
            callback(value);
        }
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
    fn create() -> HexEntry {
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
}


pub trait RGBEntryInterface {
    fn create() -> Self;
}

pub struct RGBHexEntryBoxData {
    hbox: gtk::Box,
    red_entry: HexEntry,
    green_entry: HexEntry,
    blue_entry: HexEntry,
    callbacks: RefCell<Vec<Box<Fn(RGB)>>>
}

impl_widget_wrapper!(RGBHexEntryBoxData, hbox, gtk::Box);

impl RGBHexEntryBoxData {
    pub fn get_rgb(&self) -> RGB {
        let max_value = u16::max_value() as f64;
        let red = self.red_entry.get_value() as f64 / max_value;
        let green = self.green_entry.get_value() as f64 / max_value;
        let blue = self.blue_entry.get_value() as f64 / max_value;
        RGB::from((red, green, blue))
    }

    pub fn set_rgb(&self, rgb: RGB) {
        let max_value = u16::max_value() as f64;
        let red = (rgb.red * max_value) as u32;
        let green = (rgb.green * max_value) as u32;
        let blue = (rgb.blue * max_value) as u32;
        self.red_entry.set_value(red);
        self.green_entry.set_value(green);
        self.blue_entry.set_value(blue);
    }

    pub fn connect_value_changed<F: 'static + Fn(RGB)>(&self, callback: F) {
        self.callbacks.borrow_mut().push(Box::new(callback))
    }

    fn inform_value_changed(&self) {
        let rgb = self.get_rgb();
        for callback in self.callbacks.borrow().iter() {
            callback(rgb);
        }
    }
}

pub type RGBHexEntryBox = Rc<RGBHexEntryBoxData>;

impl RGBEntryInterface for RGBHexEntryBox {
    fn create() -> RGBHexEntryBox {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
        let max_value = u16::max_value() as u32;
        let red_label = gtk::Label::new("Red");
        red_label.set_widget_colour_rgb(RED);
        let red_entry = HexEntry::create_with_max(max_value);
        let green_label = gtk::Label::new("Green");
        green_label.set_widget_colour_rgb(GREEN);
        let green_entry = HexEntry::create_with_max(max_value);
        let blue_label = gtk::Label::new("Blue");
        blue_label.set_widget_colour_rgb(BLUE);
        let blue_entry = HexEntry::create_with_max(max_value);
        hbox.pack_start(&red_label, true, true, 0);
        hbox.pack_start(&red_entry.pwo(), true, true, 0);
        hbox.pack_start(&green_label, true, true, 0);
        hbox.pack_start(&green_entry.pwo(), true, true, 0);
        hbox.pack_start(&blue_label, true, true, 0);
        hbox.pack_start(&blue_entry.pwo(), true, true, 0);
        let callbacks: RefCell<Vec<Box<Fn(RGB)>>> = RefCell::new(Vec::new());
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
}

// FILEPATH COMPLETION

pub trait PathCompletion: gtk::EntryExt + gtk::EditableSignals {
    fn _enable_path_completion(&self, dirs_only: bool) {
        let entry_completion = gtk::EntryCompletion::new();
        entry_completion.pack_start(&gtk::CellRendererText::new(), true);
        entry_completion.set_text_column(0);
        entry_completion.set_inline_completion(true);
        entry_completion.set_inline_selection(true);
        entry_completion.set_minimum_key_length(0);
        let list_store = gtk::ListStore::new(&[gtk::Type::String]);
        entry_completion.set_model(Some(&list_store.clone()));

        self.set_completion(&entry_completion);
        self.connect_changed(
            move |editable| {
                let dir_path_txt = match editable.get_text() {
                    Some(text) => pw_pathux::dir_path_text(&text).to_string(),
                    None => "".to_string()
                };
                list_store.clear();
                let dir_path = pw_pathux::expand_home_dir_or_mine(&PathBuf::from(&dir_path_txt));
                let abs_dir_path = pw_pathux::absolute_path_buf(&dir_path);
                if let Ok(entries) = pw_pathux::usable_dir_entries(&abs_dir_path) {
                    if dirs_only {
                        for entry in entries.iter() {
                            if !entry.is_dir() {
                                continue
                            };
                            let text = dir_path_txt.clone() + &entry.file_name();
                            list_store.append_row(&vec![text.to_value()]);
                        }
                    } else {
                        let msep = format!("{}", MAIN_SEPARATOR);
                        for entry in entries.iter() {
                            let text = if entry.is_dir() {
                                dir_path_txt.clone() + &entry.file_name() + &msep
                            } else {
                                dir_path_txt.clone() + &entry.file_name()
                            };
                            list_store.append_row(&vec![text.to_value()]);
                        }
                    }
                };
            }
        );
    }

    fn enable_dir_path_completion(&self) {
        self._enable_path_completion(true)
    }

    fn enable_file_path_completion(&self) {
        self._enable_path_completion(false)
    }
}

impl PathCompletion for gtk::Entry {}

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
