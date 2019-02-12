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

use std::clone::Clone;
use std::collections::HashMap;
use std::ops::BitOr;

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
    widgets: Vec<W>,
    is_on: bool,
}

impl<W> ConditionalWidgetGroup<W>
where
    W: WidgetExt + Clone + PartialEq,
{
    fn new(wsc: WidgetStatesControlled) -> ConditionalWidgetGroup<W> {
        ConditionalWidgetGroup::<W> {
            widget_states_controlled: wsc,
            widgets: Vec::new(),
            is_on: false,
        }
    }

    fn contains_widget(&self, widget: &W) -> bool {
        self.widgets.contains(widget)
    }

    fn add_widget(&mut self, widget: W) {
        match self.widget_states_controlled {
            Sensitivity => widget.set_sensitive(self.is_on),
            Visibility => widget.set_visible(self.is_on),
            Both => {
                widget.set_sensitive(self.is_on);
                widget.set_visible(self.is_on);
            }
        }
        self.widgets.push(widget.clone());
    }

    fn set_state(&mut self, on: bool) {
        match self.widget_states_controlled {
            Sensitivity => {
                for widget in self.widgets.iter() {
                    widget.set_sensitive(on);
                }
            }
            Visibility => {
                for widget in self.widgets.iter() {
                    widget.set_visible(on);
                }
            }
            Both => {
                for widget in self.widgets.iter() {
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
#[derive(Debug)]
pub struct ConditionalWidgetGroups<W>
where
    W: WidgetExt + Clone + PartialEq,
{
    widget_states_controlled: WidgetStatesControlled,
    groups: HashMap<u64, ConditionalWidgetGroup<W>>,
    current_condns: u64,
}

impl<W> ConditionalWidgetGroups<W>
where
    W: WidgetExt + Clone + PartialEq,
{
    pub fn new(wsc: WidgetStatesControlled) -> ConditionalWidgetGroups<W> {
        ConditionalWidgetGroups::<W> {
            widget_states_controlled: wsc,
            groups: HashMap::new(),
            current_condns: 0,
        }
    }

    fn contains_widget(&self, widget: &W) -> bool {
        for group in self.groups.values() {
            if group.contains_widget(widget) {
                return true;
            }
        }
        false
    }

    pub fn add_widget(&mut self, widget: W, condns: u64) {
        assert!(!self.contains_widget(&widget));
        if let Some(group) = self.groups.get_mut(&condns) {
            group.add_widget(widget);
            return;
        }
        let mut group = ConditionalWidgetGroup::<W>::new(self.widget_states_controlled);
        group.set_state((condns & self.current_condns) == condns);
        group.add_widget(widget);
        self.groups.insert(condns, group);
    }

    pub fn update_condns(&mut self, changed_condns: MaskedCondns) {
        assert!(changed_condns.is_consistent());
        let new_condns = changed_condns.condns | (self.current_condns & !changed_condns.mask);
        for (key_condns, group) in self.groups.iter_mut() {
            if changed_condns.mask & key_condns != 0 {
                group.set_state((key_condns & new_condns) == *key_condns);
            };
        }
        self.current_condns = new_condns
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
