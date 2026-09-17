// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

pub use gtk;
pub use gtk::atk;
pub use gtk::gdk;
pub use gtk::gdk_pixbuf;
pub use gtk::gio;
pub use gtk::glib;

pub use gtk::cairo;

pub use pango;
pub use pango_sys;
pub use pangocairo;

pub use recollections;
pub use sourceview;

#[macro_use]
pub mod gtkx;

pub mod gdk_pixbufx;
pub mod gdkx;
pub mod geometry;
// pub mod glibx;
pub mod printer;
pub mod sample;
pub mod sav_state;
#[macro_use]
pub mod wrapper;

pub static UNEXPECTED: &str = "Unexpected error: please inform <pwil3058@bigpond.net.au>";

#[macro_export]
macro_rules! yield_to_pending_events {
    ( ) => {
        while gtk::events_pending() {
            gtk::main_iteration();
        }
    };
}
