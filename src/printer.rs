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

use std::fs;
use std::path;

use gdk_pixbuf;
use gtk;
use gtk::prelude::{IsA, PrintSettingsExt};

use mut_static::*;

use dialogue;

struct RememberedPrinterSettings {
    pub o_file_path: Option<path::PathBuf>,
}

impl RememberedPrinterSettings {
    fn set_file_path(&mut self, file_path: &path::Path) {
        if !file_path.exists() {
            if let Some(dir_path) = file_path.parent() {
                if !dir_path.exists() {
                    fs::create_dir_all(&dir_path).unwrap_or_else(
                        |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                    );
                }
            };
            if let Err(err) = gtk::PrintSettings::new().to_file(file_path) {
                panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
            };
        };
        self.o_file_path = Some(file_path.to_path_buf())
    }
}

lazy_static! {
    static ref REMEMBERED_PRINTER_SETTINGS: MutStatic<RememberedPrinterSettings> = {
        MutStatic::from(RememberedPrinterSettings{o_file_path: None})
    };
}

pub fn init_printer(file_path: &path::Path) {
    REMEMBERED_PRINTER_SETTINGS.write().unwrap().set_file_path(file_path);
}

pub
fn get_printer_settings() -> gtk::PrintSettings {
    let settings = gtk::PrintSettings::new();
    if let Some(ref file_path) = REMEMBERED_PRINTER_SETTINGS.write().unwrap().o_file_path {
        if let Err(err) = settings.load_file(file_path) {
            panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
        }
    };
    settings
}

pub
fn save_printer_settings(settings: &gtk::PrintSettings) {
    if let Some(ref file_path) = REMEMBERED_PRINTER_SETTINGS.write().unwrap().o_file_path {
        if let Err(err) = settings.to_file(file_path) {
            panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
        }
    };
}

// TODO: finish implementing printing
pub fn print_pixbuf<P: IsA<gtk::Window>>(_pixbuf: &gdk_pixbuf::Pixbuf, parent: Option<&P>) {
    let prop = gtk::PrintOperation::new();
    dialogue::inform_user(parent, "Printing not yet implemented", None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
