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

use gdk;
use gtk;
use gtk::prelude::*;

use crate::colour::*;
use crate::rgb_math::rgb::*;

#[allow(deprecated)]
pub trait Colourable: gtk::WidgetExt {
    fn set_widget_colour(&self, colour: &Colour) {
        self.set_widget_colour_rgb(colour.rgb())
    }

    fn set_widget_colour_rgb(&self, rgb: RGB) {
        let bg_rgba = gdk::RGBA::from(rgb);
        let fg_rgba = gdk::RGBA::from(rgb.best_foreground_rgb());
        self.override_background_color(gtk::StateFlags::empty(), Some(&bg_rgba));
        self.override_color(gtk::StateFlags::empty(), Some(&fg_rgba));
    }
}

#[allow(deprecated)]
impl Colourable for gtk::Button {
    fn set_widget_colour_rgb(&self, rgb: RGB) {
        let bg_rgba = gdk::RGBA::from(rgb);
        let fg_rgba = gdk::RGBA::from(rgb.best_foreground_rgb());
        self.override_background_color(gtk::StateFlags::empty(), Some(&bg_rgba));
        self.override_color(gtk::StateFlags::empty(), Some(&fg_rgba));
        for child in self.get_children().iter() {
            child.set_widget_colour_rgb(rgb);
        }
    }
}

impl Colourable for gtk::Arrow {}
impl Colourable for gtk::Bin {}
impl Colourable for gtk::Box {}
impl Colourable for gtk::ButtonBox {}
impl Colourable for gtk::CheckButton {}
impl Colourable for gtk::ComboBox {}
impl Colourable for gtk::ComboBoxText {}
impl Colourable for gtk::Container {}
impl Colourable for gtk::Entry {}
impl Colourable for gtk::EventBox {}
impl Colourable for gtk::FlowBox {}
impl Colourable for gtk::Frame {}
impl Colourable for gtk::Grid {}
impl Colourable for gtk::Label {}
impl Colourable for gtk::LinkButton {}
impl Colourable for gtk::MenuBar {}
impl Colourable for gtk::RadioButton {}
impl Colourable for gtk::Scrollbar {}
impl Colourable for gtk::SpinButton {}
impl Colourable for gtk::ToggleButton {}
impl Colourable for gtk::ToolButton {}
impl Colourable for gtk::Toolbar {}
impl Colourable for gtk::Widget {}
