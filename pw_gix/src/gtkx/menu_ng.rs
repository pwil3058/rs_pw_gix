// Copyright 2018 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gdk;
use gtk;
use gtk::prelude::*;

use crate::{sav_state::*, wrapper::*};

#[derive(PWO)]
pub struct ManagedMenu {
    menu: gtk::Menu,
    items: Rc<ConditionalWidgets<&'static str, gtk::MenuItem>>,
}

impl ManagedMenu {
    pub fn menu(&self) -> gtk::Menu {
        self.menu.clone()
    }

    pub fn menu_item(&self, name: &'static str) -> Option<gtk::MenuItem> {
        self.items.get_widget(&name)
    }

    fn append_menu_item(&self, name: &'static str, item: &gtk::MenuItem, condns: u64) {
        self.items.add_widget(name, item, condns);
        self.menu.append(item);
        self.menu.show_all();
    }

    fn insert_menu_item(
        &self,
        name: &'static str,
        item: &gtk::MenuItem,
        condns: u64,
        position: i32,
    ) {
        self.items.add_widget(name, item, condns);
        self.menu.insert(item, position);
        self.menu.show_all();
    }

    fn prepend_menu_item(&self, name: &'static str, item: &gtk::MenuItem, condns: u64) {
        self.items.add_widget(name, item, condns);
        self.menu.prepend(item);
        self.menu.show_all();
    }

    fn new_item(
        &self,
        label_text: &str,
        image: Option<&gtk::Image>,
        tooltip_text: &str,
    ) -> gtk::MenuItem {
        let item = gtk::MenuItem::new();
        let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        if let Some(image) = image {
            h_box.pack_start(image, false, false, 0);
        }
        let label = gtk::Label::new(Some(label_text));
        label.set_xalign(0.0);
        h_box.pack_start(&label, true, true, 0);
        item.add(&h_box);
        item.set_tooltip_text(Some(tooltip_text));

        item
    }

    pub fn append_item(
        &self,
        name: &'static str,
        label_text: &str,
        image: Option<&gtk::Image>,
        tooltip_text: &str,
        condns: u64,
    ) -> gtk::MenuItem {
        let item = self.new_item(label_text, image, tooltip_text);
        self.append_menu_item(name, &item, condns);

        item
    }

    pub fn insert_item(
        &self,
        name: &'static str,
        label_text: &str,
        image: Option<&gtk::Image>,
        tooltip_text: &str,
        condns: u64,
        position: i32,
    ) -> gtk::MenuItem {
        let item = self.new_item(label_text, image, tooltip_text);
        self.insert_menu_item(name, &item, condns, position);

        item
    }

    pub fn prepend_item(
        &self,
        name: &'static str,
        label_text: &str,
        image: Option<&gtk::Image>,
        tooltip_text: &str,
        condns: u64,
    ) -> gtk::MenuItem {
        let item = self.new_item(label_text, image, tooltip_text);
        self.prepend_menu_item(name, &item, condns);

        item
    }

    pub fn append_separator(&self) {
        self.menu.append(&gtk::SeparatorMenuItem::new());
        self.menu.show_all();
    }

    pub fn insert_separator(&self, position: i32) {
        self.menu.insert(&gtk::SeparatorMenuItem::new(), position);
        self.menu.show_all();
    }

    pub fn prepend_separator(&self) {
        self.menu.prepend(&gtk::SeparatorMenuItem::new());
        self.menu.show_all();
    }

    pub fn update_condns(&self, changed_condns: MaskedCondns) {
        self.items.update_condns(changed_condns)
    }

    pub fn update_hover_condns(&self, hover_ok: bool) {
        self.items.update_hover_condns(hover_ok)
    }

    pub fn popup_at_event(&self, event: &gdk::EventButton) {
        if self.items.len() > 0 {
            self.menu.popup_easy(event.get_button(), event.get_time());
        }
    }
}

pub struct ManagedMenuBuilder<'c> {
    conditional_widgets_builder: ConditionalWidgetsBuilder,
    items: &'c [(&'static str, &'c str, Option<&'c gtk::Image>, &'c str, u64)],
}

impl<'c> ManagedMenuBuilder<'c> {
    pub fn new() -> Self {
        Self {
            conditional_widgets_builder: ConditionalWidgetsBuilder::new(),
            items: &[],
        }
    }

    pub fn widget_states_controlled(
        &mut self,
        widget_states_controlled: WidgetStatesControlled,
    ) -> &mut Self {
        self.conditional_widgets_builder
            .widget_states_controlled(widget_states_controlled);
        self
    }

    pub fn selection(&mut self, selection: &gtk::TreeSelection) -> &mut Self {
        self.conditional_widgets_builder.selection(selection);
        self
    }

    pub fn change_notifier(&mut self, change_notifier: &Rc<ChangedCondnsNotifier>) -> &mut Self {
        self.conditional_widgets_builder
            .change_notifier(change_notifier);
        self
    }

    pub fn items(
        &mut self,
        items: &'c [(&'static str, &'c str, Option<&'c gtk::Image>, &'c str, u64)],
    ) -> &Self {
        self.items = items;
        self
    }

    pub fn build(&self) -> ManagedMenu {
        let menu = gtk::MenuBuilder::new().build();
        let items = self
            .conditional_widgets_builder
            .build::<&'static str, gtk::MenuItem>();
        let mm = ManagedMenu { menu, items };
        for &(name, label_text, image, tooltip_text, condns) in self.items.iter() {
            mm.append_item(name, label_text, image, tooltip_text, condns);
        }
        mm.menu.show_all();

        mm
    }
}
