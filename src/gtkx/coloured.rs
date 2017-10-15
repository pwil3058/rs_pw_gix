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

use rgb_math::rgb::*;

pub trait Colourable: gtk::WidgetExt {
    fn set_widget_colour_rgb(&self, rgb: RGB) {
        let bg_rgba = gdk::RGBA::from(rgb);
        let fg_rgba = gdk::RGBA::from(rgb.best_foreground_rgb());
        self.override_background_color(gtk::StateFlags::empty(), Some(&bg_rgba));
        self.override_color(gtk::StateFlags::empty(), Some(&fg_rgba));
    }
}

impl Colourable for gtk::Label {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
