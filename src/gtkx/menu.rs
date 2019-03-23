// Copyright 2018 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gdk;
use gtk;
use gtk::prelude::*;

use crate::sav_state::*;

pub struct WrappedMenu {
    menu: gtk::Menu,
    items: RefCell<HashMap<String, gtk::MenuItem>>,
}

impl WrappedMenu {
    pub fn new(items: &Vec<(&str, &str, &str)>) -> WrappedMenu {
        let pm = WrappedMenu {
            menu: gtk::Menu::new(),
            items: RefCell::new(HashMap::<String, gtk::MenuItem>::new()),
        };
        for &(name, label_text, tooltip_text) in items.iter() {
            pm.append_item(name, label_text, tooltip_text);
        }
        pm.menu.show_all();

        pm
    }

    pub fn len(&self) -> usize {
        self.items.borrow().len()
    }

    pub fn menu(&self) -> gtk::Menu {
        self.menu.clone()
    }

    pub fn menu_item(&self, name: &str) -> Option<gtk::MenuItem> {
        if let Some(item) = self.items.borrow().get(name) {
            Some(item.clone())
        } else {
            None
        }
    }

    pub fn append_menu_item(&self, name: &str, item: &gtk::MenuItem) {
        if let Some(_) = self
            .items
            .borrow_mut()
            .insert(name.to_string(), item.clone())
        {
            panic!("Duplicate popup menu item name: {}", name);
        };
        self.menu.append(item);
        self.menu.show_all();
    }

    pub fn insert_menu_item(&self, name: &str, item: &gtk::MenuItem, position: i32) {
        if let Some(_) = self
            .items
            .borrow_mut()
            .insert(name.to_string(), item.clone())
        {
            panic!("Duplicate popup menu item name: {}", name);
        };
        self.menu.insert(item, position);
        self.menu.show_all();
    }

    pub fn prepend_menu_item(&self, name: &str, item: &gtk::MenuItem) {
        if let Some(_) = self
            .items
            .borrow_mut()
            .insert(name.to_string(), item.clone())
        {
            panic!("Duplicate popup menu item name: {}", name);
        };
        self.menu.prepend(item);
        self.menu.show_all();
    }

    fn new_item(&self, label_text: &str, tooltip_text: &str) -> gtk::MenuItem {
        let item = gtk::MenuItem::new_with_label(label_text);
        item.set_tooltip_text(tooltip_text);

        item
    }

    pub fn append_item(&self, name: &str, label_text: &str, tooltip_text: &str) -> gtk::MenuItem {
        let item = self.new_item(label_text, tooltip_text);
        self.append_menu_item(name, &item);

        item
    }

    pub fn insert_item(
        &self,
        name: &str,
        label_text: &str,
        tooltip_text: &str,
        position: i32,
    ) -> gtk::MenuItem {
        let item = self.new_item(label_text, tooltip_text);
        self.insert_menu_item(name, &item, position);

        item
    }

    pub fn prepend_item(&self, name: &str, label_text: &str, tooltip_text: &str) -> gtk::MenuItem {
        let item = self.new_item(label_text, tooltip_text);
        self.prepend_menu_item(name, &item);

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

    pub fn connect_item_activate<F: Fn(&gtk::MenuItem) + 'static>(&self, name: &str, f: F) {
        if let Some(item) = self.items.borrow().get(name) {
            item.connect_activate(f);
        } else {
            panic!("Unknown popup menu item name: {}", name);
        }
    }

    pub fn set_sensitivities(&self, sensitivity: bool, names: &[&str]) {
        for name in names.iter() {
            if let Some(item) = self.items.borrow().get(*name) {
                item.set_sensitive(sensitivity);
            } else {
                panic!("Unknown popup menu item name: {}", name);
            }
        }
    }

    pub fn set_visibilities(&self, visibility: bool, names: &[&str]) {
        for name in names.iter() {
            if let Some(item) = self.items.borrow().get(*name) {
                item.set_visible(visibility);
            } else {
                panic!("Unknown popup menu item name: {}", name);
            }
        }
    }

    pub fn popup_at_event(&self, event: &gdk::EventButton) {
        if self.len() > 0 {
            self.menu.popup_easy(event.get_button(), event.get_time());
        }
    }
}

pub struct ManagedMenu {
    menu: gtk::Menu,
    items: Rc<ConditionalWidgetGroups<gtk::MenuItem>>,
}

impl ManagedMenu {
    pub fn new(
        wsc: WidgetStatesControlled,
        selection: Option<&gtk::TreeSelection>,
        change_notifier: Option<&Rc<ChangedCondnsNotifier>>,
        items: &Vec<(&str, &str, Option<&gtk::Image>, &str, u64)>,
    ) -> Self {
        let pm = Self {
            menu: gtk::Menu::new(),
            items: ConditionalWidgetGroups::<gtk::MenuItem>::new(wsc, selection, change_notifier),
        };
        for &(name, label_text, image, tooltip_text, condns) in items.iter() {
            pm.append_item(name, label_text, image, tooltip_text, condns);
        }
        pm.menu.show_all();

        pm
    }

    pub fn menu(&self) -> gtk::Menu {
        self.menu.clone()
    }

    pub fn menu_item(&self, name: &str) -> Option<gtk::MenuItem> {
        self.items.get_widget(name)
    }

    fn append_menu_item(&self, name: &str, item: &gtk::MenuItem, condns: u64) {
        self.items.add_widget(name, item, condns);
        self.menu.append(item);
        self.menu.show_all();
    }

    fn insert_menu_item(&self, name: &str, item: &gtk::MenuItem, condns: u64, position: i32) {
        self.items.add_widget(name, item, condns);
        self.menu.insert(item, position);
        self.menu.show_all();
    }

    fn prepend_menu_item(&self, name: &str, item: &gtk::MenuItem, condns: u64) {
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
        let label = gtk::Label::new(label_text);
        label.set_xalign(0.0);
        h_box.pack_start(&label, true, true, 0);
        item.add(&h_box);
        item.set_tooltip_text(tooltip_text);

        item
    }

    pub fn append_item(
        &self,
        name: &str,
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
        name: &str,
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
        name: &str,
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

    pub fn popup_at_event(&self, event: &gdk::EventButton) {
        if self.items.len() > 0 {
            self.menu.popup_easy(event.get_button(), event.get_time());
        }
    }
}

pub struct DualManagedMenu {
    menu: gtk::Menu,
    sensitivity: Rc<ConditionalWidgetGroups<gtk::MenuItem>>,
    visibility: Rc<ConditionalWidgetGroups<gtk::MenuItem>>,
}

impl DualManagedMenu {
    pub fn new(
        selection: Option<&gtk::TreeSelection>,
        change_notifier: Option<&Rc<ChangedCondnsNotifier>>,
        items: &Vec<(&str, &str, Option<&gtk::Image>, &str, u64, u64)>,
    ) -> Self {
        let pm = Self {
            menu: gtk::Menu::new(),
            sensitivity: ConditionalWidgetGroups::<gtk::MenuItem>::new(
                WidgetStatesControlled::Sensitivity,
                selection,
                change_notifier,
            ),
            visibility: ConditionalWidgetGroups::<gtk::MenuItem>::new(
                WidgetStatesControlled::Visibility,
                selection,
                change_notifier,
            ),
        };
        for &(name, label_text, image, tooltip_text, sensitivity_condns, visibility_condns) in
            items.iter()
        {
            pm.append_item(
                name,
                label_text,
                image,
                tooltip_text,
                sensitivity_condns,
                visibility_condns,
            );
        }
        pm.menu.show_all();

        pm
    }

    pub fn menu(&self) -> gtk::Menu {
        self.menu.clone()
    }

    pub fn menu_item(&self, name: &str) -> Option<gtk::MenuItem> {
        self.sensitivity.get_widget(name)
    }

    fn append_menu_item(
        &self,
        name: &str,
        item: &gtk::MenuItem,
        sensitivity_condns: u64,
        visibility_condns: u64,
    ) {
        self.sensitivity.add_widget(name, item, sensitivity_condns);
        self.visibility.add_widget(name, item, visibility_condns);
        self.menu.append(item);
        self.menu.show_all();
    }

    fn insert_menu_item(
        &self,
        name: &str,
        item: &gtk::MenuItem,
        sensitivity_condns: u64,
        visibility_condns: u64,
        position: i32,
    ) {
        self.sensitivity.add_widget(name, item, sensitivity_condns);
        self.visibility.add_widget(name, item, visibility_condns);
        self.menu.insert(item, position);
        self.menu.show_all();
    }

    fn prepend_menu_item(
        &self,
        name: &str,
        item: &gtk::MenuItem,
        sensitivity_condns: u64,
        visibility_condns: u64,
    ) {
        self.sensitivity.add_widget(name, item, sensitivity_condns);
        self.visibility.add_widget(name, item, visibility_condns);
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
        if let Some(image) = image {
            let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            h_box.pack_start(image, false, false, 0);
            h_box.pack_start(&gtk::Label::new(label_text), false, false, 0);
            item.add(&h_box);
        } else {
            item.set_label(label_text)
        }
        item.set_tooltip_text(tooltip_text);

        item
    }

    pub fn append_item(
        &self,
        name: &str,
        label_text: &str,
        image: Option<&gtk::Image>,
        tooltip_text: &str,
        sensitivity_condns: u64,
        visibility_condns: u64,
    ) -> gtk::MenuItem {
        let item = self.new_item(label_text, image, tooltip_text);
        self.append_menu_item(name, &item, sensitivity_condns, visibility_condns);

        item
    }

    pub fn insert_item(
        &self,
        name: &str,
        label_text: &str,
        image: Option<&gtk::Image>,
        tooltip_text: &str,
        sensitivity_condns: u64,
        visibility_condns: u64,
        position: i32,
    ) -> gtk::MenuItem {
        let item = self.new_item(label_text, image, tooltip_text);
        self.insert_menu_item(name, &item, sensitivity_condns, visibility_condns, position);

        item
    }

    pub fn prepend_item(
        &self,
        name: &str,
        label_text: &str,
        image: Option<&gtk::Image>,
        tooltip_text: &str,
        sensitivity_condns: u64,
        visibility_condns: u64,
    ) -> gtk::MenuItem {
        let item = self.new_item(label_text, image, tooltip_text);
        self.prepend_menu_item(name, &item, sensitivity_condns, visibility_condns);

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
        self.sensitivity.update_condns(changed_condns);
        self.visibility.update_condns(changed_condns);
    }

    pub fn popup_at_event(&self, event: &gdk::EventButton) {
        if self.sensitivity.len() > 0 {
            self.menu.popup_easy(event.get_button(), event.get_time());
        }
    }
}
