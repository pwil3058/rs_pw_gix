// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use crate::gdk;
use crate::glib::{self, wrapper};
use crate::gtk::{self, prelude::*, subclass::prelude::*};

use crate::gtkx::coloured::ColourableWidgetExt;

#[derive(Default)]
pub struct PlacardImp;

#[glib::object_subclass]
impl ObjectSubclass for PlacardImp {
    const NAME: &str = "Placard";
    type Type = Placard;
    type ParentType = gtk::Button;
}

impl ObjectImpl for PlacardImp {
    fn constructed(&self) {
        self.parent_constructed();

        // Make sure we have a label child
        self.obj().set_label("");
        self.obj().set_relief(gtk::ReliefStyle::None);
        self.obj().set_focus_on_click(false);
        self.obj().set_can_focus(false);
    }
}
impl WidgetImpl for PlacardImp {}
impl ContainerImpl for PlacardImp {}
impl BinImpl for PlacardImp {}
impl ButtonImpl for PlacardImp {}

wrapper! {
    pub struct Placard(ObjectSubclass<PlacardImp>)
        @extends gtk::Button, gtk::Widget, gtk::Container, gtk::Bin;
}

impl Placard {
    pub fn builder() -> PlacardBuilder {
        PlacardBuilder::default()
    }

    pub fn new() -> Self {
        glib::Object::builder::<Placard>().build()
    }

    pub fn set_label_bold(&self, label: &str) {
        self.set_markup(format!("<b>{}</b>", label).as_str());
    }

    fn set_markup(&self, markup: &str) {
        self.children().iter().for_each(|child| {
            if let Some(label) = child.downcast_ref::<gtk::Label>() {
                label.set_markup(markup);
            }
        })
    }
}

impl Default for Placard {
    fn default() -> Self {
        Self::new()
    }
}

impl ColourableWidgetExt for Placard {}

#[derive(Default)]
pub struct PlacardBuilder {
    bold: bool,
    text: String,
    colours: Option<(gdk::RGBA, gdk::RGBA)>,
}

impl PlacardBuilder {
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn label(&mut self, label: &str) -> &mut PlacardBuilder {
        self.text = label.to_owned();
        self
    }

    pub fn colours(
        &mut self,
        background: &gdk::RGBA,
        foreground: &gdk::RGBA,
    ) -> &mut PlacardBuilder {
        self.colours = Some((*background, *foreground));
        self
    }

    pub fn build(&self) -> Placard {
        let placard = Placard::default();

        if self.bold {
            placard.set_label_bold(&self.text);
        } else {
            placard.set_label(&self.text);
        }
        if let Some((background, foreground)) = self.colours {
            placard.set_widget_colours(&background, &foreground);
        }

        placard
    }
}

#[cfg(test)]
mod placard_tests {
    use super::*;
    use gtk::Label;

    use crate::gdk::RGBA;

    #[test]
    fn test_new() {
        // Initialize GTK context for testing
        gtk::init().expect("Failed to initialize GTK");

        let placard = Placard::new();
        placard.set_label("label");
        placard.set_widget_colours(
            &RGBA::new(1.0, 0.0, 0.0, 1.0),
            &RGBA::new(0.0, 1.0, 0.0, 1.0),
        );
        let placard2 = Placard::builder().label("label").build();
        debug_assert_eq!(placard.label(), placard2.label());
        placard2.children().iter().for_each(|child| {
            if let Some(label) = child.downcast_ref::<Label>() {
                debug_assert_eq!(Some(label.label()), placard2.label());
            }
        })
    }
}
