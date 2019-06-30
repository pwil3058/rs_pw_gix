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
use gdk::WindowExt;
use gdk_pixbuf::Pixbuf;
use glib;
pub use glib::Cast;
use gtk;
use gtk::prelude::*;

pub use crate::gtkx::dialog::dialog_user::{parent_none, DialogUser, TopGtkWindow, CANCEL_OK_BUTTONS};
use crate::printer::*;

#[macro_export]
macro_rules! define_gtkw_using_pwo {
    () => (
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
    )
}

#[macro_export]
macro_rules! impl_widget_wrapper {
    ( $field:ident: $t:ty, $f:ident ) => (
        impl PackableWidgetObject for $f {
            type PWT = $t;

            fn pwo(&self) -> $t {
                self.$field.clone()
            }
        }

        impl TopGtkWindow for $f {
            define_gtkw_using_pwo!();
        }

        impl DialogUser for $f {}

        impl WidgetWrapper for $f {}
    );
    ( $field:ident: $t:ty, $f:ident < $( $g:ident ),+ > $( $constraints:tt )*) => (
        impl<$($g),*> PackableWidgetObject for $f<$($g),*>
            $($constraints)*
        {
            type PWT = $t;

            fn pwo(&self) -> $t {
                self.$field.clone()
            }
        }

        impl<$($g),*> TopGtkWindow for $f<$($g),*>
            $($constraints)*
        {
            define_gtkw_using_pwo!();
        }

        impl<$($g),*> DialogUser for $f<$($g),*> $($constraints)* {}

        impl<$($g),*> WidgetWrapper for $f<$($g),*> $($constraints)* {}
    );
    ( $($i:ident).+() -> $t:ty, $f:ident ) => (
        impl PackableWidgetObject for $f {
            type PWT = $t;

            fn pwo(&self) -> $t {
                self.$($i).*()
            }
        }

        impl TopGtkWindow for $f {
            define_gtkw_using_pwo!();
        }

        impl DialogUser for $f {}

        impl WidgetWrapper for $f {}
    );
    ( $($i:ident).+() -> $t:ty, $f:ident < $( $g:ident ),+ > $( $constraints:tt )*) => (
        impl<$($g),*> PackableWidgetObject for $f<$($g),*>
            $($constraints)*
        {
            type PWT = $t;

            fn pwo(&self) -> $t {
                self.$($i).*()
            }
        }

        impl<$($g),*> TopGtkWindow for $f<$($g),*>
            $($constraints)*
        {
            define_gtkw_using_pwo!();
        }

        impl<$($g),*> DialogUser for $f<$($g),*> $($constraints)* {}

        impl<$($g),*> WidgetWrapper for $f<$($g),*> $($constraints)* {}
    );
}

pub trait PackableWidgetObject {
    type PWT: glib::IsA<gtk::Widget> + WidgetExt;

    fn pwo(&self) -> Self::PWT;
}

pub enum CursorSpec<'a> {
    Type(gdk::CursorType),
    Name(&'a str),
    Pixbuf((&'a Pixbuf, i32, i32)),
}

pub trait WidgetWrapper: PackableWidgetObject + DialogUser {
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
            gdk::Cursor::new_from_name(display, name)
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

    fn show_busy(&self) -> Option<gdk::Cursor> {
        let o_old_cursor = self.get_cursor();
        self.set_cursor_from_spec(CursorSpec::Type(gdk::CursorType::Watch));
        yield_to_pending_events!();
        o_old_cursor
    }

    fn unshow_busy(&self, o_cursor: Option<gdk::Cursor>) {
        if let Some(cursor) = o_cursor {
            self.set_cursor(Some(&cursor));
        } else {
            self.set_cursor(None);
        }
        yield_to_pending_events!();
    }

    fn do_showing_busy<F: 'static + Fn(&Self)>(&self, action: F) {
        let o_old_cursor = self.show_busy();
        action(self);
        self.unshow_busy(o_old_cursor);
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
        where
            A: Eq,
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

    #[test]
    fn widget_wrapper_complex_generic_constrained() {
        trait Alpha<B>
        where
            B: PartialEq,
        {
        }
        struct _TestWrapper<A, B>
        where
            A: Alpha<B>,
            B: PartialEq,
        {
            vbox: gtk::Box,
            a: A,
            b: B,
        }

        impl_widget_wrapper!(vbox: gtk::Box, _TestWrapper<A, B>
            where
                A: Alpha<B>,
                B: PartialEq,
        );
    }
}
