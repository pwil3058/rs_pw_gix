// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use glib::Cast;
use gtk::WidgetExt;

pub trait TopGtkWindow {
    fn get_toplevel_gtk_window(&self) -> Option<gtk::Window>;
}

macro_rules! implement_tgw_for_widget {
    ( $f:ident ) => {
        impl TopGtkWindow for gtk::$f {
            fn get_toplevel_gtk_window(&self) -> Option<gtk::Window> {
                if let Some(widget) = self.get_toplevel() {
                    if widget.is_toplevel() {
                        if let Ok(window) = widget.dynamic_cast::<gtk::Window>() {
                            return Some(window);
                        }
                    }
                };
                None
            }
        }
    };
}

implement_tgw_for_widget!(Bin);
implement_tgw_for_widget!(Box);
implement_tgw_for_widget!(Container);
implement_tgw_for_widget!(DrawingArea);
implement_tgw_for_widget!(Entry);
implement_tgw_for_widget!(EventBox);
implement_tgw_for_widget!(Frame);
implement_tgw_for_widget!(Grid);
implement_tgw_for_widget!(Layout);
implement_tgw_for_widget!(ListBox);
implement_tgw_for_widget!(Notebook);
implement_tgw_for_widget!(Paned);
implement_tgw_for_widget!(ScrolledWindow);
implement_tgw_for_widget!(Stack);
implement_tgw_for_widget!(TextView);
implement_tgw_for_widget!(TreeView);
implement_tgw_for_widget!(Widget);
implement_tgw_for_widget!(Window);
implement_tgw_for_widget!(ApplicationWindow);
implement_tgw_for_widget!(Dialog);
implement_tgw_for_widget!(AboutDialog);
implement_tgw_for_widget!(AppChooserDialog);
implement_tgw_for_widget!(ColorChooserDialog);
implement_tgw_for_widget!(FileChooserDialog);
implement_tgw_for_widget!(FontChooserDialog);
implement_tgw_for_widget!(MessageDialog);
implement_tgw_for_widget!(RecentChooserDialog);

pub trait DialogUser: TopGtkWindow {}
