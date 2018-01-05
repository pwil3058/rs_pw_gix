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

use std::path::{PathBuf};

use gtk;
use gtk::prelude::*;

use pw_pathux;

use gtkx::entry::*;

pub fn parent_none() -> Option<&'static gtk::Window> {
    let none: Option<&gtk::Window> = None;
    none
}

// INFORM
pub fn create_inform_user_dialog<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    msg: &str,
    expln: Option<&str>,
    problem_type: gtk::MessageType
) -> gtk::MessageDialog {
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
    dialog
}

pub fn inform_user<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    msg: &str,
    expln: Option<&str>,
) {
    create_inform_user_dialog(dialog_parent, msg, expln, gtk::MessageType::Info).run();
}

pub fn warn_user<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    msg: &str,
    expln: Option<&str>,
) {
    create_inform_user_dialog(dialog_parent, msg, expln, gtk::MessageType::Warning).run();
}

// ASK QUESTION
pub fn ask_question<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    question: &str,
    expln: Option<&str>,
    buttons: &[(&str, i32)],
) -> i32 {
    let dialog = gtk::MessageDialog::new(
        dialog_parent,
        gtk::DialogFlags::empty(),
        gtk::MessageType::Question,
        gtk::ButtonsType::None,
        question,
    );
    for button in buttons {
        dialog.add_button(button.0, button.1);
    }
    if let Some(explanation) = expln {
        dialog.set_property_secondary_text(Some(explanation));
    };
    dialog.run()
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

// PATHS
pub fn select_file<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    prompt: Option<&str>,
    suggestion: Option<&str>,
    existing: bool,
    absolute: bool,
) -> Option<PathBuf> {
    let action = if existing {
        gtk::FileChooserAction::Open
    } else {
        gtk::FileChooserAction::Save
    };
    let dialog = gtk::FileChooserDialog::new(prompt, dialog_parent, action);
    dialog.add_button("gtk-cancel", gtk::ResponseType::Cancel.into());
    dialog.add_button("gtk-ok", gtk::ResponseType::Ok.into());
    let ok = gtk::ResponseType::Ok;
    let ok = ok.into();
    dialog.set_default_response(ok);
    if let Some(suggestion) = suggestion {
        dialog.set_filename(suggestion);
    };
    if dialog.run() == ok {
        if let Some(file_path) = dialog.get_filename() {
            dialog.destroy();
            if absolute {
                return Some(pw_pathux::absolute_path_buf(&file_path));
            } else {
                return Some(pw_pathux::relative_path_buf_or_mine(&file_path));
            }
        };
    };
    dialog.destroy();
    None
}

pub fn ask_file_path<P: IsA<gtk::Window>>(
    dialog_parent: Option<&P>,
    prompt: Option<&str>,
    suggestion: Option<&str>,
    existing: bool,
) -> Option<PathBuf> {
    let dialog = gtk::Dialog::new_with_buttons(
        None,
        dialog_parent,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        &[("gtk-cancel", gtk::ResponseType::Cancel.into()), ("gtk-ok", gtk::ResponseType::Ok.into())]
    );
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
    dialog.get_content_area().pack_start(&hbox, false, false, 0);

    let prompt_label = if prompt.is_some() {
        gtk::Label::new(prompt)
    } else {
        gtk::Label::new(Some("File Path:"))
    };
    hbox.pack_start(&prompt_label, false, false, 0);

    let entry = gtk::Entry::new();
    entry.enable_file_path_completion();
    entry.set_activates_default(true);
    entry.set_width_chars(32);
    if let Some(suggestion) = suggestion {
        entry.set_text(suggestion)
    };
    hbox.pack_start(&entry, true, true, 0);

    let button = gtk::Button::new_with_label("Browse");
    hbox.pack_start(&button, false, false, 0);
    hbox.show_all();
    let b_prompt = if let Some(prompt_text) = prompt {
        format!("Select {}", prompt_text)
    } else {
        "Select File Path:".to_string()
    };
    let entry_c = entry.clone();
    let dialog_c = dialog.clone();
    button.connect_clicked(
        move |_| {
            // NB: following gymnastics need to satisfy lifetime  checks
            let text = &entry_c.get_text().unwrap_or("".to_string());
            let suggestion: Option<&str> = if text.len() > 0 { Some(text) } else { None };
            if let Some(file_path) = select_file(Some(&dialog_c), Some(&b_prompt), suggestion, existing, false) {
                let text = pw_pathux::path_to_string(&file_path);
                entry_c.set_text(&text);
            }
        }
    );

    let ok = gtk::ResponseType::Ok;
    let ok = ok.into();
    dialog.set_default_response(ok);
    if dialog.run() == ok {
        dialog.close();
        if let Some(text) = entry.get_text() {
            Some(PathBuf::from(text))
        } else {
            Some(PathBuf::new())
        }
    } else {
        dialog.close();
        None
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
