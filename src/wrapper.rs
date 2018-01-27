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

use std::error::Error;
use std::path::{PathBuf};

use gdk;
use gdk::WindowExt;
use gdk_pixbuf::Pixbuf;
use glib;
use gtk;
use gtk::prelude::*;

use pw_pathux;

use gtkx::dialog::*;
use gtkx::entry::*;
use printer::*;

#[macro_export]
macro_rules! impl_widget_wrapper {
    ( $field:ident: $t:ty, $f:ident ) => (
        impl WidgetWrapper for $f {
            type PWT = $t;

            fn pwo(&self) -> $t {
                self.$field.clone()
            }
        }
    );
    ( $field:ident: $t:ty, $f:ident < $( $g:ident ),+ > $( $constraints:tt )*) => (
        impl<$($g),*> WidgetWrapper for $f<$($g),*>
            $($constraints)*
        {
            type PWT = $t;

            fn pwo(&self) -> $t {
                self.$field.clone()
            }
        }
    );
    ( $($i:ident).+() -> $t:ty, $f:ident ) => (
        impl WidgetWrapper for $f {
            type PWT = $t;

            fn pwo(&self) -> $t {
                self.$($i).*()
            }
        }
    );
    ( $($i:ident).+() -> $t:ty, $f:ident < $( $g:ident ),+ > $( $constraints:tt )*) => (
        impl<$($g),*> WidgetWrapper for $f<$($g),*>
            $($constraints)*
        {
            type PWT = $t;

            fn pwo(&self) -> $t {
                self.$($i).*()
            }
        }
    );
}

pub fn parent_none() -> Option<&'static gtk::Window> {
    let none: Option<&gtk::Window> = None;
    none
}

pub enum CursorSpec<'a> {
    Type(gdk::CursorType),
    Name(&'a str),
    Pixbuf((&'a Pixbuf, i32, i32)),
}

pub static CANCEL_OK_BUTTONS: &[(&str, i32)] = &[("Cancel", 0), ("Ok", 1)];

pub trait WidgetWrapper {
    type PWT: glib::IsA<gtk::Widget> + WidgetExt;

    fn pwo(&self) -> Self::PWT;

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

    fn set_transient_for_and_icon_on<W: gtk::GtkWindowExt>(&self, window: &W) {
        if let Some(tlw) = self.get_toplevel_gtk_window() {
            window.set_transient_for(Some(&tlw));
            if let Some(ref icon) = tlw.get_icon() {
                window.set_icon(Some(icon));
            }
        };
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

    fn new_dialog(&self) -> gtk::Dialog {
        let dialog = gtk::Dialog::new();
        self.set_transient_for_and_icon_on(&dialog);
        dialog
    }

    fn new_dialog_with_buttons(
        &self,
        title: Option<&str>,
        flags: gtk::DialogFlags,
        buttons: &[(&str, i32)]
    )  -> gtk::Dialog {
        let dialog = gtk::Dialog::new_with_buttons(title, parent_none(), flags, buttons);
        self.set_transient_for_and_icon_on(&dialog);
        dialog
    }

    fn new_message_dialog(
        &self,
        flags: gtk::DialogFlags,
        type_: gtk::MessageType,
        buttons: &[(&str, i32)],
        message: &str,
    ) -> gtk::MessageDialog {
        let dialog = gtk::MessageDialog::new(parent_none(), flags, type_, gtk::ButtonsType::None, message);
        for button in buttons {
            dialog.add_button(button.0, button.1);
        }
        self.set_transient_for_and_icon_on(&dialog);
        dialog
    }

    fn new_inform_user_dialog(
        &self,
        msg: &str,
        o_expln: Option<&str>,
        problem_type: gtk::MessageType
    ) -> gtk::MessageDialog {
        let buttons = &[("Close", 0),];
        let dialog = self.new_message_dialog(gtk::DialogFlags::empty(), problem_type, buttons, msg);
        if let Some(expln) = o_expln {
            dialog.set_property_secondary_text(Some(expln));
        };
        dialog.enable_auto_close();
        dialog
    }

    fn inform_user(&self, msg: &str, o_expln: Option<&str>) {
        self.new_inform_user_dialog(msg, o_expln, gtk::MessageType::Info).run();
    }

    fn warn_user(&self, msg: &str, o_expln: Option<&str>) {
        self.new_inform_user_dialog(msg, o_expln, gtk::MessageType::Warning).run();
    }

    fn report_error<E: Error>(&self, msg: &str, error: &E) {
        let mut expln = error.description().to_string();
        if let Some(cause) = error.cause() {
            expln += &format!("\nCaused by: {}.", cause.description());
        };
        self.new_inform_user_dialog( msg, Some(&expln), gtk::MessageType::Error).run();
    }

    fn ask_question(&self, question: &str, o_expln: Option<&str>, buttons: &[(&str, i32)],) -> i32 {
        let dialog = self.new_message_dialog(gtk::DialogFlags::empty(), gtk::MessageType::Question, buttons, question);
        if let Some(expln) = o_expln {
            dialog.set_property_secondary_text(Some(expln));
        };
        dialog.enable_auto_close();
        dialog.run()
    }

    fn ask_confirm_action(&self, msg: &str, expln: Option<&str>) -> bool {
        self.ask_question(msg, expln, CANCEL_OK_BUTTONS) == 1
    }

    fn new_file_chooser_dialog(
        &self,
        o_title: Option<&str>,
        action: gtk::FileChooserAction,
    ) -> gtk::FileChooserDialog {
        let dialog = gtk::FileChooserDialog::new(o_title, parent_none(), action);
        self.set_transient_for_and_icon_on(&dialog);
        dialog
    }

    fn browse_path(
        &self,
        o_prompt: Option<&str>,
        o_suggestion: Option<&str>,
        action: gtk::FileChooserAction,
        absolute: bool,
    ) -> Option<PathBuf> {
        let dialog = self.new_file_chooser_dialog(o_prompt, action);
        for button in CANCEL_OK_BUTTONS {
            dialog.add_button(button.0, button.1);
        }
        let ok = CANCEL_OK_BUTTONS[1].1;
        dialog.set_default_response(ok);
        if let Some(suggestion) = o_suggestion {
            dialog.set_filename(suggestion);
        };
        if dialog.run() == ok {
            if let Some(file_path) = dialog.get_filename() {
                dialog.destroy();
                if absolute {
                    return Some(pw_pathux::absolute_path_buf(&file_path));
                } else {
                    return Some(pw_pathux::relative_path_buf_or_mine(&file_path));
                }
            };
        };
        dialog.destroy();
        None
    }

    fn ask_path(
        &self,
        prompt: Option<&str>,
        suggestion: Option<&str>,
        action: gtk::FileChooserAction,
    ) -> Option<PathBuf> {
        let dialog = self.new_dialog_with_buttons(None, gtk::DialogFlags::DESTROY_WITH_PARENT,CANCEL_OK_BUTTONS);
        dialog.connect_close(
            |d| d.destroy()
        );
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
        dialog.get_content_area().pack_start(&hbox, false, false, 0);

        let prompt_label = if prompt.is_some() {
            gtk::Label::new(prompt)
        } else {
            gtk::Label::new(Some("File Path:"))
        };
        hbox.pack_start(&prompt_label, false, false, 0);

        let entry = gtk::Entry::new();
        match action {
            gtk::FileChooserAction::Open | gtk::FileChooserAction::Save => {
                entry.enable_file_path_completion();
            },
            gtk::FileChooserAction::SelectFolder | gtk::FileChooserAction::CreateFolder => {
                entry.enable_dir_path_completion();
            },
            _ => panic!("File: {} Line: {}: must specify a (useful) action"),
        };
        entry.set_activates_default(true);
        entry.set_width_chars(32);
        if let Some(suggestion) = suggestion {
            entry.set_text(suggestion)
        };
        hbox.pack_start(&entry, true, true, 0);

        let button = gtk::Button::new_with_label("Browse");
        hbox.pack_start(&button, false, false, 0);
        hbox.show_all();
        let b_prompt = if let Some(prompt_text) = prompt {
            format!("Select {}", prompt_text)
        } else {
            "Select Path:".to_string()
        };
        let entry_c = entry.clone();
        let dialog_c = dialog.clone();
        button.connect_clicked(
            move |_| {
                // NB: following gymnastics need to satisfy lifetime  checks
                let text = &entry_c.get_text().unwrap_or("".to_string());
                let suggestion: Option<&str> = if text.len() > 0 { Some(text) } else { None };
                if let Some(path) = browse_path(Some(&dialog_c), Some(&b_prompt), suggestion, action, false) {
                    let text = pw_pathux::path_to_string(&path);
                    entry_c.set_text(&text);
                }
            }
        );

        let ok = CANCEL_OK_BUTTONS[1].1;
        dialog.set_default_response(ok);
        if dialog.run() == ok {
            let o_text = entry.get_text();
            dialog.destroy();
            if let Some(text) = o_text {
                Some(PathBuf::from(text))
            } else {
                Some(PathBuf::new())
            }
        } else {
            dialog.destroy();
            None
        }
    }

    fn select_dir(&self, o_prompt: Option<&str>, o_suggestion: Option<&str>, existing: bool, absolute: bool) -> Option<PathBuf> {
        if existing {
            self.browse_path(o_prompt, o_suggestion, gtk::FileChooserAction::SelectFolder, absolute)
        } else {
            self.browse_path(o_prompt, o_suggestion, gtk::FileChooserAction::CreateFolder, absolute)
        }
    }

    fn select_file(&self, o_prompt: Option<&str>, o_suggestion: Option<&str>, existing: bool, absolute: bool) -> Option<PathBuf> {
        if existing {
            self.browse_path(o_prompt, o_suggestion, gtk::FileChooserAction::Open, absolute)
        } else {
            self.browse_path(o_prompt, o_suggestion, gtk::FileChooserAction::Save, absolute)
        }
    }

    fn ask_dir_path(&self, o_prompt: Option<&str>, o_suggestion: Option<&str>, existing: bool) -> Option<PathBuf> {
        if existing {
            self.ask_path(o_prompt, o_suggestion, gtk::FileChooserAction::SelectFolder)
        } else {
            self.ask_path(o_prompt, o_suggestion, gtk::FileChooserAction::CreateFolder)
        }
    }

    fn ask_file_path(&self, o_prompt: Option<&str>, o_suggestion: Option<&str>, existing: bool) -> Option<PathBuf> {
        if existing {
            self.ask_path(o_prompt, o_suggestion, gtk::FileChooserAction::Open)
        } else {
            self.ask_path(o_prompt, o_suggestion, gtk::FileChooserAction::Save)
        }
    }

    fn print_pixbuf(&self, pixbuf: &Pixbuf) -> PrintResult {
        if let Some(parent) = self.get_toplevel_gtk_window() {
            print_pixbuf(pixbuf, Some(&parent))
        } else {
            print_pixbuf(pixbuf, parent_none())
        }
    }

    fn print_text(&self, text: &str) -> PrintResult {
        if let Some(parent) = self.get_toplevel_gtk_window() {
            print_text(text, Some(&parent))
        } else {
            print_text(text, parent_none())
        }
    }

    fn print_markup_chunks(&self, chunks: Vec<String>) -> PrintResult {
        if let Some(parent) = self.get_toplevel_gtk_window() {
            print_markup_chunks(chunks, Some(&parent))
        } else {
            print_markup_chunks(chunks, parent_none())
        }
    }
}

fn browse_path<P: IsA<gtk::Window> + gtk::GtkWindowExt>(
    dialog_parent: Option<&P>,
    o_prompt: Option<&str>,
    o_suggestion: Option<&str>,
    action: gtk::FileChooserAction,
    absolute: bool,
) -> Option<PathBuf> {
    let dialog = gtk::FileChooserDialog::new(o_prompt, dialog_parent, action);
    if let Some(parent) = dialog_parent {
        if let Some(ref icon) = parent.get_icon() {
            dialog.set_icon(icon)
        }
    };
    for button in CANCEL_OK_BUTTONS {
        dialog.add_button(button.0, button.1);
    }
    let ok = CANCEL_OK_BUTTONS[1].1;
    dialog.set_default_response(ok);
    if let Some(suggestion) = o_suggestion {
        dialog.set_filename(suggestion);
    };
    if dialog.run() == ok {
        if let Some(file_path) = dialog.get_filename() {
            dialog.destroy();
            if absolute {
                return Some(pw_pathux::absolute_path_buf(&file_path));
            } else {
                return Some(pw_pathux::relative_path_buf_or_mine(&file_path));
            }
        };
    };
    dialog.destroy();
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk;

    #[test]
    fn widget_wrapper_simple() {
        struct _TestWrapper {
            vbox: gtk::Box,
        }

        impl_widget_wrapper!(vbox: gtk::Box, _TestWrapper);
    }

    #[test]
    fn widget_wrapper_simple_expr() {
        struct _TestWrapper {
            vbox: gtk::Box,
        }

        impl _TestWrapper {
            fn vbox(&self) -> gtk::Box {
                self.vbox.clone()
            }
        }

        impl_widget_wrapper!(vbox() -> gtk::Box, _TestWrapper);
    }

    #[test]
    fn widget_wrapper_generic_simple() {
        struct _TestWrapper<A, B, C> {
            vbox: gtk::Box,
            a: A,
            b: B,
            c: C,
        }

        impl_widget_wrapper!(vbox: gtk::Box, _TestWrapper<A, B, C>);
    }

    #[test]
    fn widget_wrapper_generic_constrained() {
        struct _TestWrapper<A, B, C>
            where   A: Eq,
                    C: PartialEq,
        {
            vbox: gtk::Box,
            a: A,
            b: B,
            c: C,
        }

        impl_widget_wrapper!(vbox: gtk::Box, _TestWrapper<A, B, C>
            where   A: Eq,
                    C: PartialEq,
        );
    }
}
