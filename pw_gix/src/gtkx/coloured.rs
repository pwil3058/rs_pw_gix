// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::prelude::*;

use crate::colour::*;

#[allow(deprecated)]
pub trait Colourable: gtk::WidgetExt {
    fn set_widget_colour(&self, colour: &Colour) {
        self.set_widget_colour_rgb(colour.rgb().into())
    }

    fn set_widget_colour_rgb(&self, rgb: RGB) {
        let bg_rgba = rgba_from_rgb(rgb);
        let fg_rgba = rgba_from_rgb(rgb.best_foreground_rgb());
        self.override_background_color(gtk::StateFlags::empty(), Some(&bg_rgba));
        self.override_color(gtk::StateFlags::empty(), Some(&fg_rgba));
    }
}

#[allow(deprecated)]
impl Colourable for gtk::Button {
    fn set_widget_colour_rgb(&self, rgb: RGB) {
        let bg_rgba = rgba_from_rgb(rgb);
        let fg_rgba = rgba_from_rgb(rgb.best_foreground_rgb());
        self.override_background_color(gtk::StateFlags::empty(), Some(&bg_rgba));
        self.override_color(gtk::StateFlags::empty(), Some(&fg_rgba));
        for child in self.get_children().iter() {
            child.set_widget_colour_rgb(rgb);
        }
    }
}

impl Colourable for gtk::Bin {}
impl Colourable for gtk::Box {}
impl Colourable for gtk::ButtonBox {}
impl Colourable for gtk::CheckButton {}
impl Colourable for gtk::ComboBox {}
impl Colourable for gtk::ComboBoxText {}
impl Colourable for gtk::Container {}
impl Colourable for gtk::Entry {}
impl Colourable for gtk::EventBox {}
impl Colourable for gtk::FlowBox {}
impl Colourable for gtk::Frame {}
impl Colourable for gtk::Grid {}
impl Colourable for gtk::Label {}
impl Colourable for gtk::LinkButton {}
impl Colourable for gtk::MenuBar {}
impl Colourable for gtk::RadioButton {}
impl Colourable for gtk::Scrollbar {}
impl Colourable for gtk::SpinButton {}
impl Colourable for gtk::ToggleButton {}
impl Colourable for gtk::ToolButton {}
impl Colourable for gtk::Toolbar {}
impl Colourable for gtk::Widget {}