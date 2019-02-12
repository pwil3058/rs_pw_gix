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
use std::default::Default;
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

#[derive(Debug)]
struct SensitiveWidgetGroup<W>
where
    W: WidgetExt + Clone,
{
    widgets: HashMap<String, W>,
    is_sensitive: bool,
}

impl<W> Default for SensitiveWidgetGroup<W>
where
    W: WidgetExt + Clone,
{
    fn default() -> Self {
        SensitiveWidgetGroup::<W> {
            widgets: HashMap::new(),
            is_sensitive: false,
        }
    }
}

impl<W> SensitiveWidgetGroup<W>
where
    W: WidgetExt + Clone,
{
    fn add_widget(&mut self, name: &str, widget: W) {
        widget.set_sensitive(self.is_sensitive);
        self.widgets.insert(name.to_string(), widget.clone());
    }

    fn get_widget(&self, name: &str) -> Option<&W> {
        self.widgets.get(name)
    }

    fn set_sensitive(&mut self, sensitive: bool) {
        for widget in self.widgets.values() {
            widget.set_sensitive(sensitive)
        }
        self.is_sensitive = sensitive
    }
}

/// Groups of widgets whose sensitivity is determined by the current
/// conditions
// TODO: get a better name than SensitiveWidgetGroups
#[derive(Debug)]
pub struct SensitiveWidgetGroups<W>
where
    W: WidgetExt + Clone,
{
    groups: HashMap<u64, SensitiveWidgetGroup<W>>,
    current_condns: u64,
}

impl<W> Default for SensitiveWidgetGroups<W>
where
    W: WidgetExt + Clone,
{
    fn default() -> Self {
        SensitiveWidgetGroups::<W> {
            groups: HashMap::new(),
            current_condns: 0,
        }
    }
}

impl<W> SensitiveWidgetGroups<W>
where
    W: WidgetExt + Clone,
{
    pub fn add_widget(&mut self, name: &str, widget: W, condns: u64) {
        for group in self.groups.values() {
            if group.widgets.contains_key(name) {
                panic!("{}: duplicate key in SensitiveWidgetGroups", name);
            }
        }
        if let Some(group) = self.groups.get_mut(&condns) {
            group.add_widget(name, widget);
            return;
        }
        let mut group = SensitiveWidgetGroup::<W>::default();
        group.add_widget(name, widget);
        self.groups.insert(condns, group);
    }

    pub fn get_widget(&self, name: &str) -> Option<&W> {
        for group in self.groups.values() {
            match group.get_widget(name) {
                Some(w) => return Some(w),
                None => (),
            }
        }
        None
    }

    pub fn update_condns(&mut self, changed_condns: MaskedCondns) {
        assert!(changed_condns.is_consistent());
        let new_condns = changed_condns.condns | (self.current_condns & !changed_condns.mask);
        for (key_condns, group) in self.groups.iter_mut() {
            if changed_condns.mask & key_condns != 0 {
                group.set_sensitive((key_condns & new_condns) == *key_condns);
            };
        }
        self.current_condns = new_condns
    }
}

#[derive(Debug)]
struct VisibleWidgetGroup<W>
where
    W: WidgetExt + Clone,
{
    widgets: HashMap<String, W>,
    is_visible: bool,
}

impl<W> Default for VisibleWidgetGroup<W>
where
    W: WidgetExt + Clone,
{
    fn default() -> Self {
        VisibleWidgetGroup::<W> {
            widgets: HashMap::new(),
            is_visible: false,
        }
    }
}

impl<W> VisibleWidgetGroup<W>
where
    W: WidgetExt + Clone,
{
    fn add_widget(&mut self, name: &str, widget: W) {
        widget.set_sensitive(self.is_visible);
        self.widgets.insert(name.to_string(), widget.clone());
    }

    fn get_widget(&self, name: &str) -> Option<&W> {
        self.widgets.get(name)
    }

    fn set_visible(&mut self, visible: bool) {
        for widget in self.widgets.values() {
            widget.set_visible(visible)
        }
        self.is_visible = visible
    }
}

#[derive(Debug)]
pub struct VisibleWidgetGroups<W>
where
    W: WidgetExt + Clone,
{
    groups: HashMap<u64, VisibleWidgetGroup<W>>,
    current_condns: u64,
}

impl<W> Default for VisibleWidgetGroups<W>
where
    W: WidgetExt + Clone,
{
    fn default() -> Self {
        VisibleWidgetGroups::<W> {
            groups: HashMap::new(),
            current_condns: 0,
        }
    }
}

impl<W> VisibleWidgetGroups<W>
where
    W: WidgetExt + Clone,
{
    pub fn add_widget(&mut self, name: &str, widget: W, condns: u64) {
        for group in self.groups.values() {
            if group.widgets.contains_key(name) {
                panic!("{}: duplicate key in VisibleWidgetGroups", name);
            }
        }
        if let Some(group) = self.groups.get_mut(&condns) {
            group.add_widget(name, widget);
            return;
        }
        let mut group = VisibleWidgetGroup::<W>::default();
        group.add_widget(name, widget);
        self.groups.insert(condns, group);
    }

    pub fn get_widget(&self, name: &str) -> Option<&W> {
        for group in self.groups.values() {
            match group.get_widget(name) {
                Some(w) => return Some(w),
                None => (),
            }
        }
        None
    }

    pub fn update_condns(&mut self, changed_condns: MaskedCondns) {
        assert!(changed_condns.is_consistent());
        let new_condns = changed_condns.condns | (self.current_condns & !changed_condns.mask);
        for (key_condns, group) in self.groups.iter_mut() {
            if changed_condns.mask & key_condns != 0 {
                group.set_visible((key_condns & new_condns) == *key_condns);
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
