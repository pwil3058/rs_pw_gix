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
//! of groups of widgets dependent on the application's current state.

use std::clone::Clone;
use std::collections::HashMap;
use std::ops::BitOr;

use gtk::{TreeSelection, TreeSelectionExt, WidgetExt};

#[derive(Debug, Clone, Copy, Default)]
pub struct MaskedCondn {
    condn: u64,
    mask: u64,
}

impl BitOr for MaskedCondn {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        MaskedCondn {
            condn: self.condn | rhs.condn,
            mask: self.mask | rhs.condn,
        }
    }
}

pub trait MaskedCondnProvider {
    fn get_masked_conditions(&self) -> MaskedCondn;
}

const _SAV_DONT_CARE: u64 = 0;
const SAV_SELN_NONE: u64 = 1;
const SAV_SELN_MADE: u64 = 2;
const SAV_SELN_UNIQUE: u64 = 4;
const SAV_SELN_PAIR: u64 = 8;
const SAV_SELN_MASK: u64 = 15;

impl MaskedCondnProvider for TreeSelection {
    fn get_masked_conditions(&self) -> MaskedCondn {
        match self.count_selected_rows() {
            0 => MaskedCondn {
                condn: SAV_SELN_NONE,
                mask: SAV_SELN_MASK,
            },
            1 => MaskedCondn {
                condn: SAV_SELN_MADE + SAV_SELN_UNIQUE,
                mask: SAV_SELN_MASK,
            },
            2 => MaskedCondn {
                condn: SAV_SELN_MADE + SAV_SELN_PAIR,
                mask: SAV_SELN_MASK,
            },
            _ => MaskedCondn {
                condn: SAV_SELN_MADE,
                mask: SAV_SELN_MASK,
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct SensitiveWidgetGroup<W>
    where W: WidgetExt + Clone
{
    widgets: HashMap<String, W>,
    is_sensitive: bool,
}

impl<W> SensitiveWidgetGroup<W>
    where W: WidgetExt + Clone
{
    pub fn add_widget(&mut self, name: &str, widget: W) {
        widget.set_sensitive(self.is_sensitive);
        self.widgets.insert(name.to_string(), widget.clone());
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
