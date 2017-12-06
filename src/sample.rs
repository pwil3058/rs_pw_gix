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

use std::process::Command;

use gtk;
use gtk::{ButtonExt, WidgetExt};

use which::which;

use dialogue;

pub fn screen_sampling_available() -> bool {
    which("gnome-screenshot").is_ok()
}

pub fn take_screen_sample() {
    match Command::new("gnome-screenshot").arg("-ac").spawn() {
        Ok(_) => (),
        Err(err) => {
            let none: Option<&gtk::Window> = None;
            let explanation = format!("{:?}", err);
            dialogue::warn_user(none, "Screen sampling failed", Some(&explanation))
        }
    }
}

pub fn new_screen_sample_button(label: &str, tooltip_text: &str) -> gtk::Button {
    let btn = gtk::Button::new_with_label(label);
    if tooltip_text.len() > 0 {
        btn.set_tooltip_text(tooltip_text)
    }
    btn.connect_clicked(
        |_| take_screen_sample()
    );
    btn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
