// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use crate::gtk::{self, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, gdk::RGBA, prelude::*};

#[allow(deprecated)]
pub trait ColourableWidgetExt: WidgetExt {
    fn set_widget_colours(&self, bg: &RGBA, fg: &RGBA) {
        let background = bg.to_string();
        let foreground = fg.to_string();

        let style_context = self.style_context();
        let css = format!(
            "button {{ background-image: none; background-color: {background}; color: {foreground}; }}"
        );
        let provider = CssProvider::new();
        provider
            .load_from_data(css.as_bytes())
            .expect("Failed to load CSS");
        style_context.add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}

impl ColourableWidgetExt for gtk::Bin {}
impl ColourableWidgetExt for gtk::Box {}
impl ColourableWidgetExt for gtk::Button {}
impl ColourableWidgetExt for gtk::ButtonBox {}
impl ColourableWidgetExt for gtk::CheckButton {}
impl ColourableWidgetExt for gtk::ComboBox {}
impl ColourableWidgetExt for gtk::ComboBoxText {}
impl ColourableWidgetExt for gtk::Container {}
impl ColourableWidgetExt for gtk::Entry {}
impl ColourableWidgetExt for gtk::EventBox {}
impl ColourableWidgetExt for gtk::FlowBox {}
impl ColourableWidgetExt for gtk::Frame {}
impl ColourableWidgetExt for gtk::Grid {}
impl ColourableWidgetExt for gtk::Label {}
impl ColourableWidgetExt for gtk::LinkButton {}
impl ColourableWidgetExt for gtk::MenuBar {}
impl ColourableWidgetExt for gtk::RadioButton {}
impl ColourableWidgetExt for gtk::Scrollbar {}
impl ColourableWidgetExt for gtk::SpinButton {}
impl ColourableWidgetExt for gtk::ToggleButton {}
impl ColourableWidgetExt for gtk::ToolButton {}
impl ColourableWidgetExt for gtk::Toolbar {}
impl ColourableWidgetExt for gtk::Widget {}
