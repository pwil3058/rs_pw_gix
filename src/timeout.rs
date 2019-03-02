// Copyright 2019 Peter Williams <pwil3058@gmail.com>
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

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::{self, CheckMenuItemExt};

pub struct ControlledTimeoutCycle {
    interval_secs: Cell<u32>,
    stopped: Cell<bool>, // Help try to stop multiple timeouts being in play
    check_menu_item: gtk::CheckMenuItem,
    callbacks: RefCell<Vec<(u32, Box<Fn()>)>>,
    next_cb_id: Cell<u32>,
}

impl ControlledTimeoutCycle {
    pub fn new(label: &str, active: bool, interval_secs: u32) -> Rc<Self> {
        let ct = Rc::new(
            Self {
                interval_secs: Cell::new(interval_secs),
                stopped: Cell::new(true),
                check_menu_item: gtk::CheckMenuItem::new_with_label(label),
                callbacks: RefCell::new(Vec::new()),
                next_cb_id: Cell::new(0),
            }
        );

        let ct_clone = Rc::clone(&ct);
        ct.check_menu_item.connect_toggled(
            move |t| if t.get_active() && ct_clone.stopped.get() {
                let interval_secs = ct_clone.interval_secs.get();
                let ct_clone_clone = Rc::clone(&ct_clone);
                gtk::timeout_add_seconds(interval_secs,
                    move || {
                        for (_, callback) in ct_clone_clone.callbacks.borrow().iter() {
                            callback()
                        }
                        ct_clone_clone.stopped.set(ct_clone_clone.check_menu_item.get_active());
                        gtk::Continue(ct_clone_clone.stopped.get())
                    }
                );
                ct_clone.stopped.set(false);
            }
        );

        // NB: do this last so that call back is active
        ct.check_menu_item.set_active(active);

        ct
    }

    pub fn check_menu_item(&self) -> gtk::CheckMenuItem {
        self.check_menu_item.clone()
    }

    pub fn set_interval_secs(&self, interval_secs: u32) {
        self.interval_secs.set(interval_secs);
    }

    pub fn get_interval_secs(&self) -> u32 {
        self.interval_secs.get()
    }

    pub fn register_callback(&self, callback: Box<Fn()>) -> u32 {
        let cb_id = self.next_cb_id.get();
        self.next_cb_id.set(cb_id + 1);
        self.callbacks.borrow_mut().push((cb_id, callback));
        cb_id
    }
}
