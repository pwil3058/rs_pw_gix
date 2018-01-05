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

use std::path::{PathBuf};

use gdk;
use gdk::WindowExt;
use gdk_pixbuf::Pixbuf;
use glib;
use gtk;
use gtk::prelude::*;

use dialogue::*;

#[macro_export]
macro_rules! impl_widget_wrapper {
    ( $f:ty, $field:ident, $t:ty ) => (
        impl WidgetWrapper<$t> for $f {
            fn pwo(&self) -> $t {
                self.$field.clone()
            }
        }
    )
}

pub enum CursorSpec<'a> {
    Type(gdk::CursorType),
    Name(&'a str),
    Pixbuf((&'a Pixbuf, i32, i32)),
}

pub trait WidgetWrapper<PWT: glib::IsA<gtk::Widget> + WidgetExt> {
    fn pwo(&self) -> PWT;

    fn get_toplevel_gtk_window(&self) -> Option<gtk::Window> {
        if let Some(widget) = self.pwo().get_toplevel() {
            if widget.is_toplevel() {
                if let Ok(window) = widget.dynamic_cast::<gtk::Window>() {
                    return Some(window)
                }
            }
        };
        None
    }

    fn get_cursor(&self) -> Option<gdk::Cursor> {
        if let Some(gdk_window) = self.pwo().get_window() {
            gdk_window.get_cursor()
        } else {
            None
        }
    }

    fn set_cursor(&self, o_cursor: Option<&gdk::Cursor>) {
        if let Some(gdk_window) = self.pwo().get_window() {
            gdk_window.set_cursor(o_cursor)
        }
    }

    fn set_cursor_from_spec(&self, spec: CursorSpec) {
        if let Some(cursor) = self.new_cursor_from_spec(spec) {
            self.set_cursor(Some(&cursor))
        }
    }

    fn new_cursor(&self, cursor_type: gdk::CursorType) -> Option<gdk::Cursor> {
        if let Some(ref display) = self.pwo().get_display() {
            Some(gdk::Cursor::new_for_display(display, cursor_type))
        } else {
            None
        }
    }

    fn new_cursor_from_name(&self, name: &str) -> Option<gdk::Cursor> {
        if let Some(ref display) = self.pwo().get_display() {
            Some(gdk::Cursor::new_from_name(display, name))
        } else {
            None
        }
    }

    fn new_cursor_from_pixbuf(&self, pixbuf: &Pixbuf, x: i32, y: i32) -> Option<gdk::Cursor> {
        if let Some(ref display) = self.pwo().get_display() {
            Some(gdk::Cursor::new_from_pixbuf(display, pixbuf, x, y))
        } else {
            None
        }
    }

    fn new_cursor_from_spec(&self, spec: CursorSpec) -> Option<gdk::Cursor> {
        match spec {
            CursorSpec::Type(cursor_type) => self.new_cursor(cursor_type),
            CursorSpec::Name(name) => self.new_cursor_from_name(name),
            CursorSpec::Pixbuf(pbd) => self.new_cursor_from_pixbuf(pbd.0, pbd.1, pbd.2),
        }
    }

    fn do_showing_busy<F: 'static + Fn(&Self)>(&self, action: F) {
        let o_old_cursor = self.get_cursor();
        self.set_cursor_from_spec(CursorSpec::Type(gdk::CursorType::Clock));
        action(self);
        if let Some(old_cursor) = o_old_cursor {
            self.set_cursor(Some(&old_cursor));
        } else {
            self.set_cursor(None);
        }
    }

    fn inform_user(&self, msg: &str, expln: Option<&str>) {
        if let Some(parent) = self.get_toplevel_gtk_window() {
            inform_user(Some(&parent), msg, expln)
        } else {
            inform_user(parent_none(), msg, expln)
        }
    }

    fn warn_user(&self, msg: &str, expln: Option<&str>) {
        if let Some(parent) = self.get_toplevel_gtk_window() {
            warn_user(Some(&parent), msg, expln)
        } else {
            warn_user(parent_none(), msg, expln)
        }
    }

    fn ask_question(&self, question: &str, expln: Option<&str>, buttons: &[(&str, i32)],) -> i32 {
        if let Some(parent) = self.get_toplevel_gtk_window() {
            ask_question(Some(&parent), question, expln, buttons)
        } else {
            ask_question(parent_none(), question, expln, buttons)
        }
    }

    fn ask_confirm_action(&self, msg: &str, expln: Option<&str>) -> bool {
        if let Some(parent) = self.get_toplevel_gtk_window() {
            ask_confirm_action(Some(&parent), msg, expln)
        } else {
            ask_confirm_action(parent_none(), msg, expln)
        }
    }

    fn select_file(&self, prompt: Option<&str>, suggestion: Option<&str>, existing: bool, absolute: bool) -> Option<PathBuf> {
        if let Some(parent) = self.get_toplevel_gtk_window() {
            select_file(Some(&parent), prompt, suggestion, existing, absolute)
        } else {
            select_file(parent_none(), prompt, suggestion, existing, absolute)
        }
    }

    fn ask_file_path(&self, prompt: Option<&str>, suggestion: Option<&str>, existing: bool) -> Option<PathBuf> {
        if let Some(parent) = self.get_toplevel_gtk_window() {
            ask_file_path(Some(&parent), prompt, suggestion, existing)
        } else {
            ask_file_path(parent_none(), prompt, suggestion, existing)
        }
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
