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

//! Provide mechanisms to control the sensitivity and/or visibility
//! of groups of widgets dependent on a widget and/or an application's
//! current state.
//! Up to 64 conditions can be used to describe a state.

use std::cell::{Cell, RefCell};
use std::clone::Clone;
use std::collections::HashMap;
use std::ops::BitOr;
use std::rc::Rc;
use std::sync::Mutex;

use gtk::{TreeSelection, TreeSelectionExt, WidgetExt};

/// A struct that enables the state of a subset of the conditions to
/// be specified withoit effecting the othet conditions.
#[derive(Debug, Clone, Copy)]
pub struct MaskedCondns {
    condns: u64,
    mask: u64,
}

impl MaskedCondns {
    pub fn is_consistent(&self) -> bool {
        self.condns & !self.mask == 0
    }
}

impl BitOr for MaskedCondns {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        MaskedCondns {
            condns: self.condns | rhs.condns,
            mask: self.mask | rhs.mask,
        }
    }
}

/// A trait that we can use to add a function to existing objects to
/// determine their states,
pub trait MaskedCondnProvider {
    fn get_masked_conditions(&self) -> MaskedCondns;
}

const _SAV_DONT_CARE: u64 = 0;
/// Interesting conditions for a TreeSelection that are useful for
/// tailoring pop up menus.
const SAV_SELN_NONE: u64 = 2 ^ 0;
const SAV_SELN_MADE: u64 = 2 ^ 1;
const SAV_SELN_UNIQUE: u64 = 2 ^ 2;
const SAV_SELN_PAIR: u64 = 2 ^ 3;
const SAV_SELN_MASK: u64 = SAV_SELN_NONE + SAV_SELN_MADE + SAV_SELN_UNIQUE + SAV_SELN_PAIR;

/// Implementation of MaskedCondnProvider for TreeSelection
impl MaskedCondnProvider for TreeSelection {
    fn get_masked_conditions(&self) -> MaskedCondns {
        match self.count_selected_rows() {
            0 => MaskedCondns {
                condns: SAV_SELN_NONE,
                mask: SAV_SELN_MASK,
            },
            1 => MaskedCondns {
                condns: SAV_SELN_MADE + SAV_SELN_UNIQUE,
                mask: SAV_SELN_MASK,
            },
            2 => MaskedCondns {
                condns: SAV_SELN_MADE + SAV_SELN_PAIR,
                mask: SAV_SELN_MASK,
            },
            _ => MaskedCondns {
                condns: SAV_SELN_MADE,
                mask: SAV_SELN_MASK,
            },
        }
    }
}

pub struct ChangedCondnsNotifier {
    callbacks: Mutex<RefCell<Vec<(u64, Box<Fn(MaskedCondns)>)>>>,
    next_token: Mutex<Cell<u64>>,
}

impl ChangedCondnsNotifier {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            callbacks: Mutex::new(RefCell::new(Vec::new())),
            next_token: Mutex::new(Cell::new(0)),
        })
    }

    pub fn register_callback(&self, callback: Box<Fn(MaskedCondns)>) -> u64 {
        let next_token = self.next_token.lock().unwrap();
        let token = next_token.get();
        next_token.set(token + 1);

        let callbacks = self.callbacks.lock().unwrap();
        callbacks.borrow_mut().push((token, callback));

        token
    }

    pub fn deregister_callback(&self, token: u64) {
        let callbacks = self.callbacks.lock().unwrap();
        let position = callbacks.borrow().iter().position(|x| x.0 == token);
        if let Some(position) = position {
            callbacks.borrow_mut().remove(position);
        }
    }

    pub fn notify_changed_condns(&self, condns: MaskedCondns) {
        let callbacks = self.callbacks.lock().unwrap();
        for (_, callback) in callbacks.borrow().iter() {
            callback(condns)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetStatesControlled {
    Sensitivity,
    Visibility,
    Both,
}

use self::WidgetStatesControlled::*;

#[derive(Debug)]
struct ConditionalWidgetGroup<W>
where
    W: WidgetExt + Clone + PartialEq,
{
    widget_states_controlled: WidgetStatesControlled,
    widgets: HashMap<String, W>,
    is_on: bool,
}

impl<W> ConditionalWidgetGroup<W>
where
    W: WidgetExt + Clone + PartialEq,
{
    fn new(wsc: WidgetStatesControlled) -> ConditionalWidgetGroup<W> {
        ConditionalWidgetGroup::<W> {
            widget_states_controlled: wsc,
            widgets: HashMap::new(),
            is_on: false,
        }
    }

    fn contains_name(&self, name: &str) -> bool {
        self.widgets.contains_key(name)
    }

    fn contains_widget(&self, widget: &W) -> bool {
        for value in self.widgets.values() {
            if value == widget {
                return true
            }
        }
        false
    }

    fn add_widget(&mut self, name: &str, widget: W) {
        match self.widget_states_controlled {
            Sensitivity => widget.set_sensitive(self.is_on),
            Visibility => widget.set_visible(self.is_on),
            Both => {
                widget.set_sensitive(self.is_on);
                widget.set_visible(self.is_on);
            }
        }
        self.widgets.insert(name.to_string(), widget.clone());
    }

    fn set_state(&mut self, on: bool) {
        match self.widget_states_controlled {
            Sensitivity => {
                for widget in self.widgets.values() {
                    widget.set_sensitive(on);
                }
            }
            Visibility => {
                for widget in self.widgets.values() {
                    widget.set_visible(on);
                }
            }
            Both => {
                for widget in self.widgets.values() {
                    widget.set_sensitive(on);
                    widget.set_visible(on);
                }
            }
        }
        self.is_on = on
    }
}

/// Groups of widgets whose sensitivity and/or visibility is determined
/// by the current conditions
// TODO: get a better name than ConditionalWidgetGroups
pub struct ConditionalWidgetGroups<W>
where
    W: WidgetExt + Clone + PartialEq,
{
    widget_states_controlled: WidgetStatesControlled,
    groups: RefCell<HashMap<u64, ConditionalWidgetGroup<W>>>,
    current_condns: Cell<u64>,
    change_notifier: Rc<ChangedCondnsNotifier>,
}

impl<W> ConditionalWidgetGroups<W>
where
    W: WidgetExt + Clone + PartialEq,
{
    pub fn new(
        wsc: WidgetStatesControlled,
        selection: Option<gtk::TreeSelection>,
        change_notifier: Option<&Rc<ChangedCondnsNotifier>>,
    ) -> Rc<ConditionalWidgetGroups<W>> {
        let change_notifier = if let Some(change_notifier) = change_notifier {
            Rc::clone(&change_notifier)
        } else {
            ChangedCondnsNotifier::new()
        };
        let cwg = Rc::new(ConditionalWidgetGroups::<W> {
            widget_states_controlled: wsc,
            groups: RefCell::new(HashMap::new()),
            current_condns: Cell::new(0),
            change_notifier: change_notifier,
        });
        if let Some(selection) = selection {
            let cwg_clone = Rc::clone(&cwg);
            selection
                .connect_changed(move |seln| cwg_clone.update_condns(seln.get_masked_conditions()));
        }
        let cwg_clone = Rc::clone(&cwg);
        cwg.change_notifier
            .register_callback(Box::new(move |condns| cwg_clone.update_condns(condns)));
        cwg
    }

    pub fn change_notifier(&self) -> &Rc<ChangedCondnsNotifier> {
        &self.change_notifier
    }

    fn contains_name(&self, name: &str) -> bool {
        for group in self.groups.borrow().values() {
            if group.contains_name(name) {
                return true;
            }
        }
        false
    }

    fn contains_widget(&self, widget: &W) -> bool {
        for group in self.groups.borrow().values() {
            if group.contains_widget(widget) {
                return true;
            }
        }
        false
    }

    pub fn add_widget(&self, name: &str, widget: W, condns: u64) {
        assert!(!self.contains_widget(&widget));
        assert!(!self.contains_name(&name));
        let mut groups = self.groups.borrow_mut();
        if let Some(group) = groups.get_mut(&condns) {
            group.add_widget(name, widget);
            return;
        }
        let mut group = ConditionalWidgetGroup::<W>::new(self.widget_states_controlled);
        group.set_state((condns & self.current_condns.get()) == condns);
        group.add_widget(name, widget);
        groups.insert(condns, group);
    }

    pub fn get_widget(&self, name: &str) -> Option<W> {
        let groups = self.groups.borrow();
        for group in groups.values() {
            if let Some(widget) = group.widgets.get(name) {
                return Some(widget.clone());
            }
        }
        None
    }

    pub fn update_condns(&self, changed_condns: MaskedCondns) {
        assert!(changed_condns.is_consistent());
        let new_condns = changed_condns.condns | (self.current_condns.get() & !changed_condns.mask);
        for (key_condns, group) in self.groups.borrow_mut().iter_mut() {
            if changed_condns.mask & key_condns != 0 {
                group.set_state((key_condns & new_condns) == *key_condns);
            };
        }
        self.current_condns.set(new_condns)
    }
}
