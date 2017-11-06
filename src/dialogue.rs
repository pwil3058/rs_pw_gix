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

use gtk;
use gtk::prelude::*;

fn low_inform_user<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    msg: &str,
    expln: Option<&str>,
    problem_type: gtk::MessageType
) {
    let dialog = gtk::MessageDialog::new(
        dialog_parent,
        gtk::DialogFlags::empty(),
        problem_type,
        gtk::ButtonsType::Close,
        msg
    );
    if let Some(explanation) = expln {
        dialog.set_property_secondary_text(Some(explanation));
    };
    dialog.connect_close(
        |d| d.destroy()
    );
    dialog.connect_response(
        |d,_| d.destroy()
    );
    dialog.run();
}

pub fn inform_user<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    msg: &str,
    expln: Option<&str>,
) {
    low_inform_user(dialog_parent, msg, expln, gtk::MessageType::Info)
}

pub fn warn_user<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    msg: &str,
    expln: Option<&str>,
) {
    low_inform_user(dialog_parent, msg, expln, gtk::MessageType::Warning)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
