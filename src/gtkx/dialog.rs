// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::io::{self, Write};

use gtk;

use crate::gdkx::*;
use crate::recollections;

pub trait AutoDestroy: gtk::DialogExt + gtk::GtkWindowExt + gtk::WidgetExt {
    fn enable_auto_destroy(&self) {
        self.connect_close(|d| d.destroy());
        self.connect_response(|d, _| d.destroy());
    }
}

impl AutoDestroy for gtk::Dialog {}
impl AutoDestroy for gtk::AboutDialog {}
impl AutoDestroy for gtk::AppChooserDialog {}
impl AutoDestroy for gtk::ColorChooserDialog {}
impl AutoDestroy for gtk::FileChooserDialog {}
impl AutoDestroy for gtk::FontChooserDialog {}
impl AutoDestroy for gtk::MessageDialog {}
impl AutoDestroy for gtk::RecentChooserDialog {}

pub mod dialog_user {
    use std::error::Error;
    use std::io;
    use std::path::PathBuf;
    use std::process;

    use glib::markup_escape_text;
    use gtk;
    use gtk::prelude::*;

    use pw_pathux;

    use super::AutoDestroy;
    use crate::gtkx::entry::PathCompletion;

    pub trait TopGtkWindow {
        fn get_toplevel_gtk_window(&self) -> Option<gtk::Window>;
    }

    macro_rules! implement_tgw_for_widget {
        ( $f:ident ) => {
            impl TopGtkWindow for gtk::$f {
                fn get_toplevel_gtk_window(&self) -> Option<gtk::Window> {
                    if let Some(widget) = self.get_toplevel() {
                        if widget.is_toplevel() {
                            if let Ok(window) = widget.dynamic_cast::<gtk::Window>() {
                                return Some(window);
                            }
                        }
                    };
                    None
                }
            }
        };
    }

    implement_tgw_for_widget!(Bin);
    implement_tgw_for_widget!(DrawingArea);
    implement_tgw_for_widget!(EventBox);
    implement_tgw_for_widget!(Frame);
    implement_tgw_for_widget!(Notebook);
    implement_tgw_for_widget!(ScrolledWindow);
    implement_tgw_for_widget!(TextView);
    implement_tgw_for_widget!(TreeView);
    implement_tgw_for_widget!(Window);
    implement_tgw_for_widget!(ApplicationWindow);

    pub fn parent_none() -> Option<&'static gtk::Window> {
        let none: Option<&gtk::Window> = None;
        none
    }

    pub static CANCEL_OK_BUTTONS: &[(&str, gtk::ResponseType)] = &[
        ("Cancel", gtk::ResponseType::Cancel),
        ("Ok", gtk::ResponseType::Ok),
    ];

    fn browse_path<P: IsA<gtk::Window> + gtk::GtkWindowExt>(
        dialog_parent: Option<&P>,
        o_prompt: Option<&str>,
        o_suggestion: Option<&str>,
        action: gtk::FileChooserAction,
        absolute: bool,
    ) -> Option<PathBuf> {
        let dialog = gtk::FileChooserDialog::new(o_prompt, dialog_parent, action);
        if let Some(parent) = dialog_parent {
            if let Some(ref icon) = parent.get_icon() {
                dialog.set_icon(Some(icon))
            }
        };
        for button in CANCEL_OK_BUTTONS {
            dialog.add_button(button.0, button.1);
        }
        dialog.set_default_response(gtk::ResponseType::Ok);
        if let Some(suggestion) = o_suggestion {
            dialog.set_filename(suggestion);
        };
        if gtk::ResponseType::from(dialog.run()) == gtk::ResponseType::Ok {
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

    fn markup_cmd_fail(cmd: &str) -> String {
        format!(
            "{}: <span foreground=\"red\">FAILED</span>",
            markup_escape_text(cmd)
        )
    }

    fn markup_cmd_warn(cmd: &str) -> String {
        format!(
            "{}: <span foreground=\"orange\">WARNED</span>",
            markup_escape_text(cmd)
        )
    }

    fn markup_cmd_ok(cmd: &str) -> String {
        format!(
            "{}: <span foreground=\"green\">OK</span>",
            markup_escape_text(cmd)
        )
    }

    fn markup_output(output: &process::Output) -> Option<String> {
        if output.stdout.len() > 0 {
            let stdout = markup_escape_text(&String::from_utf8_lossy(&output.stdout));
            if output.stderr.len() > 0 {
                let stderr = markup_escape_text(&String::from_utf8_lossy(&output.stderr));
                Some(format!(
                    "{}\n<span foreground=\"red\">{}</span>",
                    stdout, stderr
                ))
            } else {
                Some(stdout.to_string())
            }
        } else if output.stderr.len() > 0 {
            let stderr = markup_escape_text(&String::from_utf8_lossy(&output.stderr));
            Some(format!("<span foreground=\"red\">{}</span>", stderr))
        } else {
            None
        }
    }

    pub trait DialogUser: TopGtkWindow {
        fn set_transient_for_and_icon_on<W: gtk::GtkWindowExt>(&self, window: &W) {
            if let Some(tlw) = self.get_toplevel_gtk_window() {
                window.set_transient_for(Some(&tlw));
                if let Some(ref icon) = tlw.get_icon() {
                    window.set_icon(Some(icon));
                }
            };
        }

        fn new_dialog(&self) -> gtk::Dialog {
            let dialog = gtk::Dialog::new();
            self.set_transient_for_and_icon_on(&dialog);
            dialog
        }

        fn new_dialog_with_buttons(
            &self,
            title: Option<&str>,
            flags: gtk::DialogFlags,
            buttons: &[(&str, gtk::ResponseType)],
        ) -> gtk::Dialog {
            let dialog = gtk::Dialog::new_with_buttons(title, parent_none(), flags, buttons);
            self.set_transient_for_and_icon_on(&dialog);
            dialog
        }

        fn new_message_dialog(
            &self,
            flags: gtk::DialogFlags,
            type_: gtk::MessageType,
            buttons: &[(&str, gtk::ResponseType)],
            message: &str,
        ) -> gtk::MessageDialog {
            let dialog = gtk::MessageDialog::new(
                parent_none(),
                flags,
                type_,
                gtk::ButtonsType::None,
                message,
            );
            for button in buttons {
                dialog.add_button(button.0, button.1);
            }
            self.set_transient_for_and_icon_on(&dialog);
            dialog
        }

        fn new_inform_user_dialog(
            &self,
            msg: &str,
            o_expln: Option<&str>,
            problem_type: gtk::MessageType,
        ) -> gtk::MessageDialog {
            let buttons = &[("Close", gtk::ResponseType::Close)];
            let dialog =
                self.new_message_dialog(gtk::DialogFlags::empty(), problem_type, buttons, msg);
            if let Some(expln) = o_expln {
                dialog.set_property_secondary_text(Some(expln));
            };
            dialog.enable_auto_destroy();
            dialog
        }

        fn inform_user(&self, msg: &str, o_expln: Option<&str>) {
            self.new_inform_user_dialog(msg, o_expln, gtk::MessageType::Info)
                .run();
        }

        fn warn_user(&self, msg: &str, o_expln: Option<&str>) {
            self.new_inform_user_dialog(msg, o_expln, gtk::MessageType::Warning)
                .run();
        }

        fn report_error<E: Error>(&self, msg: &str, error: &E) {
            let mut expln = error.description().to_string();
            if let Some(source) = error.source() {
                expln += &format!("\nCaused by: {}.", source.description());
            };
            self.new_inform_user_dialog(msg, Some(&expln), gtk::MessageType::Error)
                .run();
        }

        fn report_command_result(&self, cmd: &str, result: &io::Result<process::Output>) {
            match result {
                Ok(output) => {
                    let dialog = self.new_message_dialog(
                        gtk::DialogFlags::empty(),
                        gtk::MessageType::Info,
                        &[("Close", gtk::ResponseType::Close)],
                        "",
                    );
                    dialog.enable_auto_destroy();
                    if !output.status.success() {
                        dialog.set_property_message_type(gtk::MessageType::Error);
                        dialog.set_markup(&markup_cmd_fail(cmd));
                    } else if output.stderr.len() > 0 {
                        dialog.set_property_message_type(gtk::MessageType::Warning);
                        dialog.set_markup(&markup_cmd_warn(cmd));
                    } else {
                        dialog.set_property_message_type(gtk::MessageType::Info);
                        dialog.set_markup(&markup_cmd_ok(cmd));
                    }
                    if let Some(markup) = markup_output(&output) {
                        dialog.set_property_secondary_use_markup(true);
                        dialog.set_property_secondary_text(Some(markup.as_str()));
                    }
                    dialog.run();
                }
                Err(err) => {
                    let msg = format!("{}: blew up!", cmd);
                    self.report_error(&msg, err)
                }
            }
        }

        fn report_any_command_problems(&self, cmd: &str, result: &io::Result<process::Output>) {
            match result {
                Ok(output) => {
                    if output.status.success() && output.stderr.len() == 0 {
                        // Nothing to report
                        return;
                    }
                    let dialog = self.new_message_dialog(
                        gtk::DialogFlags::empty(),
                        gtk::MessageType::Info,
                        &[("Close", gtk::ResponseType::Close)],
                        "",
                    );
                    dialog.enable_auto_destroy();
                    if !output.status.success() {
                        dialog.set_property_message_type(gtk::MessageType::Error);
                        dialog.set_markup(&markup_cmd_fail(cmd));
                    } else {
                        dialog.set_property_message_type(gtk::MessageType::Warning);
                        dialog.set_markup(&markup_cmd_warn(cmd));
                    }
                    if let Some(markup) = markup_output(&output) {
                        dialog.set_property_secondary_use_markup(true);
                        dialog.set_property_secondary_text(Some(markup.as_str()));
                    }
                    dialog.run();
                }
                Err(err) => {
                    let msg = format!("{}: blew up!", cmd);
                    self.report_error(&msg, err)
                }
            }
        }

        fn ask_question(
            &self,
            question: &str,
            o_expln: Option<&str>,
            buttons: &[(&str, gtk::ResponseType)],
        ) -> gtk::ResponseType {
            let dialog = self.new_message_dialog(
                gtk::DialogFlags::empty(),
                gtk::MessageType::Question,
                buttons,
                question,
            );
            if let Some(expln) = o_expln {
                dialog.set_property_secondary_text(Some(expln));
            };
            //dialog.enable_auto_destroy();
            let response = dialog.run();
            dialog.destroy();
            response
            //gtk::ResponseType::from(dialog.run())
        }

        fn ask_confirm_action(&self, msg: &str, expln: Option<&str>) -> bool {
            let response = self.ask_question(msg, expln, CANCEL_OK_BUTTONS);
            println!("response: {:?}", response);
            response == gtk::ResponseType::Ok
        }

        fn ask_string_cancel_or_ok(&self, question: &str) -> (gtk::ResponseType, Option<String>) {
            let dialog = self.new_dialog_with_buttons(
                None,
                gtk::DialogFlags::DESTROY_WITH_PARENT,
                CANCEL_OK_BUTTONS,
            );
            dialog.set_default_response(gtk::ResponseType::Ok);
            let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            h_box.pack_start(&gtk::Label::new(Some(question)), false, false, 2);
            let entry = gtk::Entry::new();
            h_box.pack_start(&entry, true, true, 2);
            dialog.get_content_area().pack_start(&h_box, true, true, 0);
            dialog.show_all();
            entry.set_activates_default(true);
            let response = gtk::ResponseType::from(dialog.run());
            dialog.destroy();
            if response == gtk::ResponseType::Ok {
                if let Some(gtext) = entry.get_text() {
                    (response, Some(gtext.to_string()))
                } else {
                    (response, None)
                }
            } else {
                (response, None)
            }
        }

        fn new_file_chooser_dialog(
            &self,
            o_title: Option<&str>,
            action: gtk::FileChooserAction,
        ) -> gtk::FileChooserDialog {
            let dialog = gtk::FileChooserDialog::new(o_title, parent_none(), action);
            self.set_transient_for_and_icon_on(&dialog);
            dialog
        }

        fn browse_path(
            &self,
            o_prompt: Option<&str>,
            o_suggestion: Option<&str>,
            action: gtk::FileChooserAction,
            absolute: bool,
        ) -> Option<PathBuf> {
            if let Some(tlw) = self.get_toplevel_gtk_window() {
                browse_path(Some(&tlw), o_prompt, o_suggestion, action, absolute)
            } else {
                browse_path(parent_none(), o_prompt, o_suggestion, action, absolute)
            }
        }

        fn ask_path(
            &self,
            prompt: Option<&str>,
            suggestion: Option<&str>,
            action: gtk::FileChooserAction,
        ) -> Option<PathBuf> {
            let dialog = self.new_dialog_with_buttons(
                None,
                gtk::DialogFlags::DESTROY_WITH_PARENT,
                CANCEL_OK_BUTTONS,
            );
            dialog.connect_close(|d| d.destroy());
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 1);
            dialog.get_content_area().pack_start(&hbox, false, false, 0);

            let prompt_label = if prompt.is_some() {
                gtk::Label::new(prompt)
            } else {
                gtk::Label::new(Some("File Path:"))
            };
            hbox.pack_start(&prompt_label, false, false, 0);

            let entry = gtk::Entry::new();
            match action {
                gtk::FileChooserAction::Open | gtk::FileChooserAction::Save => {
                    entry.enable_file_path_completion();
                }
                gtk::FileChooserAction::SelectFolder | gtk::FileChooserAction::CreateFolder => {
                    entry.enable_dir_path_completion();
                }
                _ => panic!("File: {} Line: {}: must specify a (useful) action"),
            };
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
                "Select Path:".to_string()
            };
            let entry_c = entry.clone();
            let dialog_c = dialog.clone();
            button.connect_clicked(move |_| {
                // NB: following gymnastics need to satisfy lifetime  checks
                //let text = &entry_c.get_text().unwrap_or("".to_string());
                //let suggestion: Option<&str> = if text.len() > 0 { Some(text) } else { None };
                let suggestion_str: String;
                let suggestion: Option<&str> = if let Some(gtext) = entry_c.get_text() {
                    suggestion_str = String::from(gtext);
                    Some(&suggestion_str)
                } else {
                    None
                };
                if let Some(path) =
                    browse_path(Some(&dialog_c), Some(&b_prompt), suggestion, action, false)
                {
                    let text = pw_pathux::path_to_string(&path);
                    entry_c.set_text(&text);
                }
            });

            dialog.set_default_response(gtk::ResponseType::Ok);
            if gtk::ResponseType::from(dialog.run()) == gtk::ResponseType::Ok {
                let o_text = entry.get_text();
                dialog.destroy();
                if let Some(text) = o_text {
                    Some(PathBuf::from(&String::from(text)))
                } else {
                    Some(PathBuf::new())
                }
            } else {
                dialog.destroy();
                None
            }
        }

        fn select_dir(
            &self,
            o_prompt: Option<&str>,
            o_suggestion: Option<&str>,
            existing: bool,
            absolute: bool,
        ) -> Option<PathBuf> {
            if existing {
                self.browse_path(
                    o_prompt,
                    o_suggestion,
                    gtk::FileChooserAction::SelectFolder,
                    absolute,
                )
            } else {
                self.browse_path(
                    o_prompt,
                    o_suggestion,
                    gtk::FileChooserAction::CreateFolder,
                    absolute,
                )
            }
        }

        fn select_file(
            &self,
            o_prompt: Option<&str>,
            o_suggestion: Option<&str>,
            existing: bool,
            absolute: bool,
        ) -> Option<PathBuf> {
            if existing {
                self.browse_path(
                    o_prompt,
                    o_suggestion,
                    gtk::FileChooserAction::Open,
                    absolute,
                )
            } else {
                self.browse_path(
                    o_prompt,
                    o_suggestion,
                    gtk::FileChooserAction::Save,
                    absolute,
                )
            }
        }

        fn ask_dir_path(
            &self,
            o_prompt: Option<&str>,
            o_suggestion: Option<&str>,
            existing: bool,
        ) -> Option<PathBuf> {
            if existing {
                self.ask_path(o_prompt, o_suggestion, gtk::FileChooserAction::SelectFolder)
            } else {
                self.ask_path(o_prompt, o_suggestion, gtk::FileChooserAction::CreateFolder)
            }
        }

        fn ask_file_path(
            &self,
            o_prompt: Option<&str>,
            o_suggestion: Option<&str>,
            existing: bool,
        ) -> Option<PathBuf> {
            if existing {
                self.ask_path(o_prompt, o_suggestion, gtk::FileChooserAction::Open)
            } else {
                self.ask_path(o_prompt, o_suggestion, gtk::FileChooserAction::Save)
            }
        }
    }

    impl DialogUser for gtk::Bin {}
    impl DialogUser for gtk::DrawingArea {}
    impl DialogUser for gtk::EventBox {}
    impl DialogUser for gtk::Frame {}
    impl DialogUser for gtk::Notebook {}
    impl DialogUser for gtk::ScrolledWindow {}
    impl DialogUser for gtk::TextView {}
    impl DialogUser for gtk::TreeView {}
    impl DialogUser for gtk::Window {}
    impl DialogUser for gtk::ApplicationWindow {}
}

fn get_dialog_size_corrn() -> (i32, i32) {
    if let Some(corrn) = recollections::recall("dialog::size_correction") {
        if let Ok((width, height)) = parse_geometry_size(corrn.as_str()) {
            return (width, height);
        } else {
            io::stderr()
                .write(b"Error parsing \"dialog::size_correction\"\n")
                .unwrap();
        }
    };
    (0, 0)
}

fn recall_dialog_last_size(key: &str, default: (i32, i32)) -> (i32, i32) {
    if let Some(last_size) = recollections::recall(key) {
        if let Ok((width, height)) = parse_geometry_size(last_size.as_str()) {
            let (w_corrn, h_corrn) = get_dialog_size_corrn();
            return (width + w_corrn, height + h_corrn);
        } else {
            let msg = format!("Error parsing \"{}\"\n", key);
            io::stderr().write(msg.as_bytes()).unwrap();
        }
    };
    default
}

pub trait RememberDialogSize: gtk::WidgetExt + gtk::GtkWindowExt {
    fn set_size_from_recollections(&self, dialog_name: &str, default: (i32, i32)) {
        let key = format!("{}::dialog::last_size", dialog_name);
        let (width, height) = recall_dialog_last_size(key.as_str(), default);
        self.set_default_size(width, height);
        self.connect_configure_event(move |_, event| {
            let text = format_geometry_size(event);
            recollections::remember(key.as_str(), text.as_str());
            false
        });
        self.connect_realize(|widget| {
            let (req_width, req_height) = widget.get_default_size();
            let allocation = widget.get_allocation();
            let width_corrn = if req_width > 0 {
                req_width - allocation.width
            } else {
                0
            };
            let height_corrn = if req_height > 0 {
                req_height - allocation.height
            } else {
                0
            };
            let text = format!("{}x{}", width_corrn, height_corrn);
            recollections::remember("dialog::size_correction", text.as_str())
        });
    }
}

impl RememberDialogSize for gtk::Dialog {}
impl RememberDialogSize for gtk::AboutDialog {}
impl RememberDialogSize for gtk::AppChooserDialog {}
impl RememberDialogSize for gtk::ColorChooserDialog {}
impl RememberDialogSize for gtk::FileChooserDialog {}
impl RememberDialogSize for gtk::FontChooserDialog {}
impl RememberDialogSize for gtk::MessageDialog {}
impl RememberDialogSize for gtk::RecentChooserDialog {}
