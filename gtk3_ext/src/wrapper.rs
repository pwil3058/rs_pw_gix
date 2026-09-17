// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

pub use crate::glib::Cast;

use crate::gdk;
use crate::gdk_pixbuf::Pixbuf;
use crate::glib;
use crate::gtk::prelude::*;

pub use crate::gtkx::dialog_user::*;

use crate::printer::*;
pub use gtk3_ext_derive::*;

pub trait PackableWidgetObject {
    type PWT: glib::IsA<gtk::Widget> + WidgetExt;

    fn pwo(&self) -> &Self::PWT;
}

pub enum CursorSpec<'a> {
    Type(gdk::CursorType),
    Name(&'a str),
    Pixbuf((&'a Pixbuf, i32, i32)),
}

pub trait WidgetWrapper: PackableWidgetObject + DialogUser {
    fn get_cursor(&self) -> Option<gdk::Cursor> {
        if let Some(gdk_window) = self.pwo().window() {
            gdk_window.cursor()
        } else {
            None
        }
    }

    fn set_cursor(&self, o_cursor: Option<&gdk::Cursor>) {
        if let Some(gdk_window) = self.pwo().window() {
            gdk_window.set_cursor(o_cursor)
        }
    }

    fn set_cursor_from_spec(&self, spec: CursorSpec<'_>) {
        if let Some(cursor) = self.new_cursor_from_spec(spec) {
            self.set_cursor(Some(&cursor))
        }
    }

    fn new_cursor(&self, cursor_type: gdk::CursorType) -> Option<gdk::Cursor> {
        gdk::Cursor::for_display(&self.pwo().display(), cursor_type)
    }

    fn new_cursor_from_name(&self, name: &str) -> Option<gdk::Cursor> {
        gdk::Cursor::from_name(&self.pwo().display(), name)
    }

    fn new_cursor_from_pixbuf(&self, pixbuf: &Pixbuf, x: i32, y: i32) -> gdk::Cursor {
        gdk::Cursor::from_pixbuf(&self.pwo().display(), pixbuf, x, y)
    }

    fn new_cursor_from_spec(&self, spec: CursorSpec<'_>) -> Option<gdk::Cursor> {
        match spec {
            CursorSpec::Type(cursor_type) => Some(self.new_cursor(cursor_type)?),
            CursorSpec::Name(name) => self.new_cursor_from_name(name),
            CursorSpec::Pixbuf(pbd) => Some(self.new_cursor_from_pixbuf(pbd.0, pbd.1, pbd.2)),
        }
    }

    fn show_busy(&self) -> Option<gdk::Cursor> {
        let o_old_cursor = self.get_cursor();
        self.set_cursor_from_spec(CursorSpec::Type(gdk::CursorType::Watch));
        crate::yield_to_pending_events!();
        o_old_cursor
    }

    fn unshow_busy(&self, o_cursor: Option<gdk::Cursor>) {
        if let Some(cursor) = o_cursor {
            self.set_cursor(Some(&cursor));
        } else {
            self.set_cursor(None);
        }
        crate::yield_to_pending_events!();
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
            print_pixbuf(pixbuf, Option::<&gtk::Window>::None)
        }
    }
}

#[cfg(test)]
mod wrapper_tests {
    use super::*;

    use std::rc::Rc;

    use crate::glib::Cast;
    use crate::gtk;
    // use crate::gtk::prelude::*;

    #[test]
    fn widget_wrapper_simple() {
        #[derive(PWO, Wrapper)]
        struct _TestWrapper {
            vbox: gtk::Box,
        }
    }

    #[test]
    fn widget_rc_wrapper_simple() {
        #[derive(PWO)]
        struct _TestWrapperCore {
            vbox: gtk::Dialog,
        }

        #[derive(PWO, Wrapper)]
        struct _TestWrapper(Rc<_TestWrapperCore>);
    }

    #[test]
    fn widget_wrapper_generic_simple() {
        #[derive(PWO)]
        struct _TestWrapperCore<A, B, C> {
            vbox: gtk::Box,
            _a: A,
            _b: B,
            _c: C,
        }

        #[derive(PWO, Wrapper)]
        struct _TestWrapper<A, B, C>(Rc<_TestWrapperCore<A, B, C>>);
    }

    #[test]
    fn widget_wrapper_generic_constrained() {
        #[derive(PWO)]
        struct _TestWrapperCore<A, B, C>
        where
            A: Eq,
            C: PartialEq,
        {
            vbox: gtk::Box,
            _a: A,
            _b: B,
            _c: C,
        }
        #[derive(PWO, Wrapper)]
        struct _TestWrapper<A, B, C>(Rc<_TestWrapperCore<A, B, C>>)
        where
            A: Eq,
            C: PartialEq;
    }

    #[test]
    fn widget_wrapper_complex_generic_constrained() {
        #[allow(dead_code)]
        trait Alpha<B>
        where
            B: PartialEq,
        {
        }
        #[derive(PWO, Wrapper)]
        struct _TestWrapperCore<A, B>
        where
            A: Alpha<B>,
            B: PartialEq,
        {
            vbox: gtk::Box,
            _a: A,
            _b: B,
        }
        #[derive(PWO, Wrapper)]
        struct _TestWrapper<A, B>(Rc<_TestWrapperCore<A, B>>)
        where
            A: Alpha<B>,
            B: PartialEq;
    }
}
