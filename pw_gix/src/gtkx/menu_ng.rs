// Copyright 2018 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use gdk;
use gtk;
use gtk::prelude::*;

use crate::{sav_state::*, wrapper::*};

#[derive(Debug, Clone)]
pub struct MenuItemSpec {
    label: String,
    image: Option<gtk::Image>,
    tooltip_text: Option<String>,
}

impl From<(&str, Option<gtk::Image>, Option<&str>)> for MenuItemSpec {
    fn from(tuple_: (&str, Option<gtk::Image>, Option<&str>)) -> Self {
        Self {
            label: tuple_.0.to_string(),
            image: tuple_.1,
            tooltip_text: if let Some(text) = tuple_.2 {
                Some(text.to_string())
            } else {
                None
            },
        }
    }
}

impl From<&MenuItemSpec> for gtk::MenuItem {
    fn from(menu_item_spec: &MenuItemSpec) -> Self {
        let item = gtk::MenuItem::new();
        let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        if let Some(image) = &menu_item_spec.image {
            h_box.pack_start(image, false, false, 0);
        }
        let label = gtk::Label::new(Some(&menu_item_spec.label));
        label.set_xalign(0.0);
        h_box.pack_start(&label, true, true, 0);
        item.add(&h_box);
        item.set_tooltip_text(if let Some(string) = &menu_item_spec.tooltip_text {
            Some(string)
        } else {
            None
        });

        item
    }
}

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

    pub fn append_item(
        &self,
        name: &'static str,
        menu_item_spec: &MenuItemSpec,
        condns: u64,
    ) -> gtk::MenuItem {
        let item = menu_item_spec.into();
        self.append_menu_item(name, &item, condns);

        item
    }

    pub fn insert_item(
        &self,
        name: &'static str,
        menu_item_spec: &MenuItemSpec,
        condns: u64,
        position: i32,
    ) -> gtk::MenuItem {
        let item = menu_item_spec.into();
        self.insert_menu_item(name, &item, condns, position);

        item
    }

    pub fn prepend_item(
        &self,
        name: &'static str,
        menu_item_spec: &MenuItemSpec,
        condns: u64,
    ) -> gtk::MenuItem {
        let item = menu_item_spec.into();
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

#[derive(Default)]
pub struct ManagedMenuBuilder {
    conditional_widgets_builder: ConditionalWidgetsBuilder,
    items: Vec<(&'static str, MenuItemSpec, u64)>,
}

impl ManagedMenuBuilder {
    pub fn new() -> Self {
        Self::default()
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

    pub fn items(&mut self, items: &[(&'static str, MenuItemSpec, u64)]) -> &Self {
        self.items = items.to_vec();
        self
    }

    pub fn build(&self) -> ManagedMenu {
        let menu = gtk::MenuBuilder::new().build();
        let items = self
            .conditional_widgets_builder
            .build::<&'static str, gtk::MenuItem>();
        let mm = ManagedMenu { menu, items };
        for (name, menu_item_spec, condns) in self.items.iter() {
            mm.append_item(*name, menu_item_spec, *condns);
        }
        mm.menu.show_all();

        mm
    }
}
