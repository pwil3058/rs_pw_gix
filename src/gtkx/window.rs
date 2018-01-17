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

use std::io::{self, Write};

use gtk;
use gtk::prelude::*;

use gdkx::*;
use recollections;

pub trait RememberGeometry: gtk::WidgetExt + gtk::GtkWindowExt {
    fn set_geometry_from_recollections(&self, window_name: &str, default_size: (i32, i32)) {
        let key = format!("{}::window::last_geometry", window_name);
        if let Some(last_geometry) = recollections::recall(key.as_str()) {
            if let Ok((width, height, x, y)) = parse_geometry(last_geometry.as_str()) {
                self.set_default_size(width, height);
                self.move_(x, y);
            } else {
                let msg = format!("Error parsing \"{}\"\n", key);
                io::stderr().write(msg.as_bytes()).unwrap();
                self.set_default_size(default_size.0, default_size.1)
            }
        } else {
            self.set_default_size(default_size.0, default_size.1)
        }
        self.connect_configure_event(
            move |_, event| {
                let text = format_geometry(event);
                recollections::remember(key.as_str(), text.as_str());
                false
            }
        );
    }
}

impl RememberGeometry for gtk::ApplicationWindow {}
impl RememberGeometry for gtk::Window {}

pub trait DerivedTransientFor: gtk::GtkWindowExt {
    fn set_transient_for_from<W: gtk::WidgetExt>(&self, widget: &W) {
        if let Some(tl) = widget.get_toplevel() {
            if tl.is_toplevel() {
                if let Ok(window) = tl.dynamic_cast::<gtk::Window>() {
                    self.set_transient_for(Some(&window))
                }
            }
        }
    }

    fn set_transient_for_and_icon_from<W: gtk::WidgetExt>(&self, widget: &W) {
        if let Some(tl) = widget.get_toplevel() {
            if tl.is_toplevel() {
                if let Ok(window) = tl.dynamic_cast::<gtk::Window>() {
                    self.set_transient_for(Some(&window));
                    if let Some(ref icon) = window.get_icon() {
                        self.set_icon(Some(icon));
                    }
                }
            }
        }
    }
}

impl DerivedTransientFor for gtk::ApplicationWindow {}
impl DerivedTransientFor for gtk::Window {}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
