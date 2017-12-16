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

pub fn parent_none() -> Option<&'static gtk::Window> {
    let none: Option<&gtk::Window> = None;
    none
}

// INFORM
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


// ASK OK OR CANCEL
pub fn create_ok_or_cancel_dialog<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    msg: &str,
    expln: Option<&str>,
) -> gtk::MessageDialog {
    let dialog = gtk::MessageDialog::new(
        dialog_parent,
        gtk::DialogFlags::empty(),
        gtk::MessageType::Question,
        gtk::ButtonsType::OkCancel,
        msg
    );
    if let Some(explanation) = expln {
        dialog.set_property_secondary_text(Some(explanation));
    };
    dialog
}

pub fn ask_confirm_action<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    msg: &str,
    expln: Option<&str>,
) -> bool {
    let dialog = create_ok_or_cancel_dialog(dialog_parent, msg, expln);
    let response: i32 = dialog.run();
    dialog.destroy();
    let ok = gtk::ResponseType::Ok;
    let ok: i32 = ok.into();
    response == ok
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
