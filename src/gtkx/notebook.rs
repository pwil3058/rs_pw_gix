// Copyright 2017 Peter Williams <pwil3058@gmail.com>
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
use std::rc::Rc;

use gio;
use gtk;
use gtk::{BoxExt, ButtonExt, ContainerExt, WidgetExt};

use crate::wrapper::*;

pub struct TabRemoveLabelCore {
    hbox: gtk::Box,
    remove_page_callbacks: RefCell<Vec<Box<Fn()>>>,
}

impl_widget_wrapper!(hbox: gtk::Box, TabRemoveLabelCore);

impl TabRemoveLabelCore {
    pub fn connect_remove_page<F: 'static + Fn()>(&self, callback: F) {
        self.remove_page_callbacks
            .borrow_mut()
            .push(Box::new(callback))
    }

    pub fn inform_remove_page(&self) {
        for callback in self.remove_page_callbacks.borrow().iter() {
            callback();
        }
    }
}

pub type TabRemoveLabel = Rc<TabRemoveLabelCore>;

pub trait TabRemoveLabelInterface {
    fn create(label_text: Option<&str>, tooltip_text: Option<&str>) -> TabRemoveLabel;
}

impl TabRemoveLabelInterface for TabRemoveLabel {
    fn create(label_text: Option<&str>, tooltip_text: Option<&str>) -> TabRemoveLabel {
        let trl = Rc::new(TabRemoveLabelCore {
            hbox: gtk::Box::new(gtk::Orientation::Horizontal, 0),
            remove_page_callbacks: RefCell::new(Vec::new()),
        });
        let label = gtk::Label::new(label_text);
        trl.hbox.pack_start(&label, true, true, 0);
        let button = gtk::Button::new();
        button.set_relief(gtk::ReliefStyle::None);
        button.set_focus_on_click(false);
        let icon = gio::ThemedIcon::new_with_default_fallbacks("window-close-symbolic");
        let image = gtk::Image::new_from_gicon(&icon, gtk::IconSize::Menu.into());
        image.set_tooltip_text(tooltip_text);
        button.add(&image);
        button.set_name("notebook-tab-remove-button");
        trl.hbox.pack_start(&button, false, false, 0);
        trl.hbox.show_all();
        let trl_c = trl.clone();
        button.connect_clicked(move |_| trl_c.inform_remove_page());

        trl
    }
}
