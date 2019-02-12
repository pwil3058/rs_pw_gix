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

use std::io::{self, Write};
use std::str::FromStr;

use gtk;

use recollections;

pub trait RememberPosition: gtk::WidgetExt + gtk::PanedExt {
    fn recall_last_position(&self, paned_name: &str, default: i32) -> i32 {
        let key = format!("{}::paned::last_position", paned_name);
        let key_c = key.clone();
        self.connect_property_position_notify(move |paned| {
            let position = paned.get_position();
            let text = format!("{}", position);
            recollections::remember(key_c.as_str(), text.as_str());
        });
        if let Some(last_position_str) = recollections::recall(key.as_str()) {
            if let Ok(last_position) = i32::from_str(last_position_str.as_str()) {
                return last_position;
            } else {
                let msg = format!("Error parsing \"{}\"\n", key);
                io::stderr().write(msg.as_bytes()).unwrap();
            }
        };
        default
    }

    fn set_position_from_recollections(&self, paned_name: &str, default: i32) {
        let position = self.recall_last_position(paned_name, default);
        self.set_position(position);
    }
}

impl RememberPosition for gtk::Paned {}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
