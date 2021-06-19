// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::gdk::prelude::IsA;
use crate::gtk::{BoxExt, MessageDialogExt};
use crate::sourceview::prelude::{DialogExt, GtkWindowExt};
use glib::Cast;
use gtk::WidgetExt;
use std::error::Error;

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

pub trait DialogUser: TopGtkWindow {
    // Necessary because not all dialog builders have a buttons() method
    const CLOSE_BUTTONS: [(&'static str, gtk::ResponseType); 1] =
        [("Close", gtk::ResponseType::Close)];
    const CANCEL_OK_BUTTONS: [(&'static str, gtk::ResponseType); 2] = [
        ("Cancel", gtk::ResponseType::Cancel),
        ("Ok", gtk::ResponseType::Ok),
    ];

    fn new_colour_chooser_dialog_builder(&self) -> gtk::ColorChooserDialogBuilder {
        let mut dialog_builder = gtk::ColorChooserDialogBuilder::new();
        if let Some(tlw) = self.get_toplevel_gtk_window() {
            if let Some(icon) = tlw.get_icon() {
                dialog_builder = dialog_builder.icon(&icon);
            }
            dialog_builder = dialog_builder.parent(&tlw);
        };

        dialog_builder
    }

    fn new_dialog_builder(&self) -> gtk::DialogBuilder {
        let mut dialog_builder = gtk::DialogBuilder::new();
        if let Some(tlw) = self.get_toplevel_gtk_window() {
            if let Some(icon) = tlw.get_icon() {
                dialog_builder = dialog_builder.icon(&icon);
            }
            dialog_builder = dialog_builder.parent(&tlw);
        };

        dialog_builder
    }

    fn new_file_chooser_dialog_builder(&self) -> gtk::FileChooserDialogBuilder {
        let mut dialog_builder = gtk::FileChooserDialogBuilder::new();
        if let Some(tlw) = self.get_toplevel_gtk_window() {
            if let Some(icon) = tlw.get_icon() {
                dialog_builder = dialog_builder.icon(&icon);
            }
            dialog_builder = dialog_builder.parent(&tlw);
        };

        dialog_builder
    }

    fn new_font_chooser_dialog_builder(&self) -> gtk::FontChooserDialogBuilder {
        let mut dialog_builder = gtk::FontChooserDialogBuilder::new();
        if let Some(tlw) = self.get_toplevel_gtk_window() {
            if let Some(icon) = tlw.get_icon() {
                dialog_builder = dialog_builder.icon(&icon);
            }
            dialog_builder = dialog_builder.parent(&tlw);
        };

        dialog_builder
    }

    fn new_message_dialog_builder(&self) -> gtk::MessageDialogBuilder {
        let mut dialog_builder = gtk::MessageDialogBuilder::new();
        if let Some(tlw) = self.get_toplevel_gtk_window() {
            if let Some(icon) = tlw.get_icon() {
                dialog_builder = dialog_builder.icon(&icon);
            }
            dialog_builder = dialog_builder.parent(&tlw);
        };

        dialog_builder
    }

    fn new_recent_chooser_dialog_builder(&self) -> gtk::RecentChooserDialogBuilder {
        let mut dialog_builder = gtk::RecentChooserDialogBuilder::new();
        if let Some(tlw) = self.get_toplevel_gtk_window() {
            if let Some(icon) = tlw.get_icon() {
                dialog_builder = dialog_builder.icon(&icon);
            }
            dialog_builder = dialog_builder.parent(&tlw);
        };

        dialog_builder
    }

    fn inform_user(&self, msg: &str, expln: Option<&str>) {
        let dialog = self
            .new_message_dialog_builder()
            .text(msg)
            .message_type(gtk::MessageType::Info)
            .buttons(gtk::ButtonsType::Close)
            .window_position(gtk::WindowPosition::Mouse)
            .build();
        dialog.set_property_secondary_text(expln);
        dialog.run();
        dialog.close()
    }

    fn warn_user(&self, msg: &str, expln: Option<&str>) {
        let dialog = self
            .new_message_dialog_builder()
            .text(msg)
            .message_type(gtk::MessageType::Warning)
            .buttons(gtk::ButtonsType::Close)
            .window_position(gtk::WindowPosition::Mouse)
            .build();
        dialog.set_property_secondary_text(expln);
        dialog.run();
        dialog.close()
    }

    fn report_error<E: Error>(&self, msg: &str, error: &E) {
        let mut expln = error.to_string();
        if let Some(source) = error.source() {
            expln += &format!("\nCaused by: {}.", source);
        };
        let dialog = self
            .new_message_dialog_builder()
            .text(msg)
            .secondary_text(&expln)
            .message_type(gtk::MessageType::Error)
            .buttons(gtk::ButtonsType::Close)
            .window_position(gtk::WindowPosition::Mouse)
            .build();
        dialog.run();
        dialog.close();
    }

    fn present_widget_cancel_or_ok<W: IsA<gtk::Widget>>(&self, widget: &W) -> gtk::ResponseType {
        let dialog = self
            .new_dialog_builder()
            .window_position(gtk::WindowPosition::Mouse)
            .build();
        dialog
            .get_content_area()
            .pack_start(widget, false, false, 0);
        for button in &Self::CANCEL_OK_BUTTONS {
            dialog.add_button(button.0, button.1);
        }
        dialog.show_all();
        let response = dialog.run();
        dialog.close();
        crate::yield_to_pending_events!();
        response
    }
}

impl DialogUser for gtk::Bin {}
impl DialogUser for gtk::DrawingArea {}
impl DialogUser for gtk::EventBox {}
impl DialogUser for gtk::Frame {}
impl DialogUser for gtk::Notebook {}
impl DialogUser for gtk::ScrolledWindow {}
impl DialogUser for gtk::TextView {}
impl DialogUser for gtk::TreeView {}
impl DialogUser for gtk::Window {}
impl DialogUser for gtk::ApplicationWindow {}
impl DialogUser for gtk::Dialog {}
impl DialogUser for gtk::AboutDialog {}
impl DialogUser for gtk::AppChooserDialog {}
impl DialogUser for gtk::ColorChooserDialog {}
impl DialogUser for gtk::FileChooserDialog {}
impl DialogUser for gtk::FontChooserDialog {}
impl DialogUser for gtk::MessageDialog {}
impl DialogUser for gtk::RecentChooserDialog {}
