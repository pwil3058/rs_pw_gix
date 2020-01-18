// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gdk;
use gdk::WindowExt;
use gdk_pixbuf::Pixbuf;
use glib;
pub use glib::Cast;
use gtk;
use gtk::prelude::*;

pub use crate::gtkx::dialog::dialog_user::{
    parent_none, DialogUser, TopGtkWindow, CANCEL_OK_BUTTONS,
};
use crate::printer::*;

pub use pw_gix_derive::*;

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

    fn set_cursor_from_spec(&self, spec: CursorSpec<'_>) {
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

    fn new_cursor_from_spec(&self, spec: CursorSpec<'_>) -> Option<gdk::Cursor> {
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
        #[derive(PWO, Wrapper)]
        struct _TestWrapper {
            vbox: gtk::Box,
        }
    }

    #[test]
    fn widget_wrapper_simple_expr() {
        #[derive(PWO, Wrapper)]
        struct _TestWrapper {
            vbox: gtk::Box,
        }
    }

    #[test]
    fn widget_wrapper_generic_simple() {
        #[derive(PWO, Wrapper)]
        struct _TestWrapper<A, B, C> {
            vbox: gtk::Box,
            _a: A,
            _b: B,
            _c: C,
        }
    }

    #[test]
    fn widget_wrapper_generic_constrained() {
        #[derive(PWO, Wrapper)]
        struct _TestWrapper<A, B, C>
        where
            A: Eq,
            C: PartialEq,
        {
            vbox: gtk::Box,
            _a: A,
            _b: B,
            _c: C,
        }
    }

    #[test]
    fn widget_wrapper_complex_generic_constrained() {
        trait Alpha<B>
        where
            B: PartialEq,
        {
        }
        #[derive(PWO, Wrapper)]
        struct _TestWrapper<A, B>
        where
            A: Alpha<B>,
            B: PartialEq,
        {
            vbox: gtk::Box,
            _a: A,
            _b: B,
        }
    }
}
