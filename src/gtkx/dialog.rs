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

use gdkx::*;
use recollections;

#[macro_use]
pub mod dialog_user {
    // NB: this macro will not work due to "lifetime" problems
    // caused by fact top_level() returns Option(gtk::Window)
    // instead of Option(&gtk::Window)
    // NB: this technique works for C and Python
    // TODO: check to see if this works now
    //#[macro_export]
    //macro_rules! find_parent_for_dialog {
        //( $widget:ident ) => (
            //{
                //if let Some(ref widget) = $widget.get_toplevel() {
                    //if widget.is_toplevel() {
                        //if let Ok(ref window) = widget.clone().dynamic_cast::<gtk::Window>() {
                            //Some(window)
                        //} else {
                            //None
                        //}
                    //} else {
                        //None
                    //}
                //} else {
                    //None
                //}
            //}
        //)
    //}

    use gtk;
    use gtk::prelude::*;

    pub trait TopGtkWindow: gtk::WidgetExt {
        fn get_toplevel_gtk_window(&self) -> Option<gtk::Window> {
            if let Some(widget) = self.get_toplevel() {
                if widget.is_toplevel() {
                    if let Ok(window) = widget.dynamic_cast::<gtk::Window>() {
                        return Some(window)
                    }
                }
            };
            None
        }
    }

    impl TopGtkWindow for gtk::Bin {}
    impl TopGtkWindow for gtk::DrawingArea {}
    impl TopGtkWindow for gtk::EventBox {}
    impl TopGtkWindow for gtk::TextView {}
    impl TopGtkWindow for gtk::TreeView {}
}


fn get_dialog_size_corrn() -> (i32, i32) {
    if let Some(corrn) = recollections::recall("dialog::size_correction") {
        if let Ok((width, height)) = parse_geometry_size(corrn.as_str()) {
            return (width, height);
        } else {
            io::stderr().write(b"Error parsing \"dialog::size_correction\"\n").unwrap();
        }
    };
    (0, 0)
}

fn recall_dialog_last_size(key: &str, default: (i32, i32)) -> (i32, i32) {
    if let Some(last_size) = recollections::recall(key) {
        if let Ok((width, height)) = parse_geometry_size(last_size.as_str()) {
            let (w_corrn, h_corrn) = get_dialog_size_corrn();
            return (width + w_corrn, height + h_corrn);
        } else {
            let msg = format!("Error parsing \"{}\"\n", key);
            io::stderr().write(msg.as_bytes()).unwrap();
        }
    };
    default
}

pub trait RememberDialogSize: gtk::WidgetExt + gtk::GtkWindowExt {
    fn set_size_from_recollections(&self, dialog_name: &str, default: (i32, i32)) {
        let key = format!("{}::dialog::last_size", dialog_name);
        let (width, height) = recall_dialog_last_size(key.as_str(), default);
        self.set_default_size(width, height);
        self.connect_configure_event(
            move |_, event| {
                let text = format_geometry_size(event);
                recollections::remember(key.as_str(), text.as_str());
                false
            }
        );
        self.connect_realize(
            |widget| {
                let (req_width, req_height) = widget.get_default_size();
                let allocation = widget.get_allocation();
                let width_corrn = if req_width > 0 { req_width - allocation.width } else { 0 };
                let height_corrn = if req_height > 0 { req_height - allocation.height } else { 0 };
                let text = format!("{}x{}", width_corrn, height_corrn);
                recollections::remember("dialog::size_correction", text.as_str())
            }
        );
    }
}

impl RememberDialogSize for gtk::Dialog {}
impl RememberDialogSize for gtk::AboutDialog {}
impl RememberDialogSize for gtk::AppChooserDialog {}
impl RememberDialogSize for gtk::ColorChooserDialog {}
impl RememberDialogSize for gtk::FileChooserDialog {}
impl RememberDialogSize for gtk::FontChooserDialog {}
impl RememberDialogSize for gtk::MessageDialog {}
impl RememberDialogSize for gtk::RecentChooserDialog {}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
