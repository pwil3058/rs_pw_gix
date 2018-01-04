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

use gdk;
use gtk;
use gtk::prelude::*;

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
        if let Some(_) = self.items.borrow_mut().insert(name.to_string(), item.clone()) {
            panic!("Duplicate popup menu item name: {}", name);
        };
        self.menu.append(item);
        self.menu.show_all();
    }

    pub fn insert_menu_item(&self, name: &str, item: &gtk::MenuItem, position: i32) {
        if let Some(_) = self.items.borrow_mut().insert(name.to_string(), item.clone()) {
            panic!("Duplicate popup menu item name: {}", name);
        };
        self.menu.insert(item, position);
        self.menu.show_all();
    }

    pub fn prepend_menu_item(&self, name: &str, item: &gtk::MenuItem) {
        if let Some(_) = self.items.borrow_mut().insert(name.to_string(), item.clone()) {
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

    pub fn insert_item(&self, name: &str, label_text: &str, tooltip_text: &str, position: i32) -> gtk::MenuItem {
        let item = self.new_item(label_text, tooltip_text);
        self.insert_menu_item(name, &item, position);

        item
    }

    pub fn prepend_item(&self, name: &str, label_text: &str, tooltip_text: &str) -> gtk::MenuItem {
        let item = self.new_item(label_text, tooltip_text);
        self.append_menu_item(name, &item);

        item
    }

    pub fn connect_item_activate<F: Fn(&gtk::MenuItem) + 'static>(&self, name:&str, f: F) {
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
        self.menu.popup_easy(event.get_button(), event.get_time());
    }
}


#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
