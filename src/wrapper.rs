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

    fn ask_file_path<P: IsA<gtk::Window>>(&self, prompt: Option<&str>, suggestion: Option<&str>, existing: bool) -> Option<PathBuf> {
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
