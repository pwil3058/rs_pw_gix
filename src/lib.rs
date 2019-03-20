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

//! Assist in simplifying GUI programming using the crates
//! included in the **gtk-rs** project <http://gtk-rs.org/> by providing
//! mechanisms to do common operations.

extern crate cairo;
extern crate crypto_hash;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate pango;
extern crate pangocairo;

extern crate fs2;
#[macro_use]
extern crate lazy_static;
extern crate mut_static;
extern crate num;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate which;

extern crate pw_pathux;

#[macro_export]
macro_rules! init_gtk_if_needed {
    () => {{
        if !gtk::is_initialized() {
            if let Err(err) = gtk::init() {
                panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
            };
        }
    }};
}

#[macro_export]
macro_rules! yield_to_pending_events {
    ( ) => {{
        loop {
            gtk::main_iteration();
            if !gtk::events_pending() {
                break;
            }
        }
    }};
}

#[macro_use]
pub mod fs_db;
#[macro_use]
pub mod wrapper;

pub mod cairox;
pub mod colour;
pub mod file_tree;
pub mod gdk_pixbufx;
pub mod gdkx;
pub mod geometry;
pub mod gtkx;
pub mod printer;
pub mod rgb_math;
pub mod sample;
pub mod sav_state;
pub mod timeout;

mod recollect {
    use std::collections::HashMap;
    use std::fs;
    use std::io::{self, Seek};
    use std::path;

    use fs2::FileExt;
    use serde_json;

    type RDB = HashMap<String, String>;

    pub struct Recollections {
        pub file_path: Option<path::PathBuf>,
    }

    impl Recollections {
        pub fn new(o_file_path: Option<&path::Path>) -> Recollections {
            if let Some(ref file_path) = o_file_path {
                if !file_path.exists() {
                    if let Some(dir_path) = file_path.parent() {
                        if !dir_path.exists() {
                            fs::create_dir_all(&dir_path).unwrap_or_else(|err| {
                                panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                            });
                        }
                    }
                    let file = fs::File::create(file_path).unwrap_or_else(|err| {
                        panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                    });
                    serde_json::to_writer(&file, &RDB::new()).unwrap_or_else(|err| {
                        panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                    });
                }
                Recollections {
                    file_path: Some(file_path.to_path_buf()),
                }
            } else {
                Recollections { file_path: None }
            }
        }

        pub fn set_data_file_path(&mut self, file_path: &path::Path) {
            if !file_path.exists() {
                if let Some(dir_path) = file_path.parent() {
                    if !dir_path.exists() {
                        fs::create_dir_all(&dir_path).unwrap_or_else(|err| {
                            panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                        });
                    }
                }
                let file = fs::File::create(file_path)
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                serde_json::to_writer(&file, &RDB::new())
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            };
            self.file_path = Some(file_path.to_path_buf())
        }

        pub fn recall(&self, name: &str) -> Option<String> {
            if let Some(ref file_path) = self.file_path {
                let file = fs::File::open(file_path)
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                file.lock_shared()
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                let hash_map: RDB = serde_json::from_reader(&file)
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                file.unlock()
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                match hash_map.get(name) {
                    Some(ref s) => Some(s.to_string()),
                    None => None,
                }
            } else {
                None
            }
        }

        pub fn recall_or_else(&self, name: &str, default: &str) -> String {
            match self.recall(name) {
                Some(string) => string,
                None => default.to_string(),
            }
        }

        pub fn remember(&self, name: &str, value: &str) {
            if let Some(ref file_path) = self.file_path {
                let mut file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(file_path)
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                file.lock_exclusive()
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                let mut hash_map: RDB = serde_json::from_reader(&file)
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                hash_map.insert(name.to_string(), value.to_string());
                file.seek(io::SeekFrom::Start(0))
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                file.set_len(0)
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                serde_json::to_writer(&file, &hash_map)
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                file.unlock()
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            }
        }
    }
}

pub mod recollections {
    //! Provide a mechanism for widgets to remember configuration
    //! data (size, position, etc.) from one session to the next.
    use mut_static::*;
    use recollect::*;
    use std::path;

    lazy_static! {
        static ref RECOLLECTIONS: MutStatic<Recollections> =
            { MutStatic::from(Recollections::new(None)) };
    }

    /// Initialise the mechanism by providing the path of the file
    /// where the data should be stored.  This would normally be a
    /// hidden file in the user's home directory or a hidden configuration
    /// directory for the application.
    ///
    /// This function should normally be called early in the application's
    /// `main()` function e.g.
    ///
    /// ```no_run
    /// fn main() {
    ///     use std::env;
    ///     use pw_gix::recollections;
    ///
    ///     let home_dir = env::home_dir().expect("badly designed OS");
    ///     recollections::init(&home_dir.join(".this_apps_recollections"));
    /// }
    /// ```
    ///
    /// If this initialisation is not performed then calls to `recall()`
    /// will return `None`, calls to `recall_or_else()` will return the
    /// default supplied and calls to `remember()` will be ignored.
    /// The operation of the application will not be effected otherwise.
    pub fn init(file_path: &path::Path) {
        RECOLLECTIONS.write().unwrap().set_data_file_path(file_path);
    }

    /// Return the `String` value associated with the given `name` or
    /// `None` if `pw_gix::recollections` has not been initialised or
    /// asked remember data associated with the given `name`.
    pub fn recall(name: &str) -> Option<String> {
        RECOLLECTIONS.read().unwrap().recall(name)
    }

    /// Return the `String` value associated with the given `name` or
    /// `default` if `pw_gix::recollections` has not been initialised or
    /// asked remember data associated with the given `name`.
    pub fn recall_or_else(name: &str, default: &str) -> String {
        RECOLLECTIONS.read().unwrap().recall_or_else(name, default)
    }

    /// Remember the string specified by `value` and associate it with
    /// the given `name` for later recall.
    pub fn remember(name: &str, value: &str) {
        RECOLLECTIONS.read().unwrap().remember(name, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path;

    #[test]
    fn recollect_test() {
        let recollection_file = path::Path::new(".recollection_test");
        let recollections = recollect::Recollections::new(Some(&recollection_file));
        assert_eq!(recollections.recall("anything"), None);
        assert_eq!(recollections.recall_or_else("anything", "but"), "but");
        recollections.remember("anything", "whatever");
        assert_eq!(
            recollections.recall("anything"),
            Some("whatever".to_string())
        );
        assert_eq!(recollections.recall_or_else("anything", "but"), "whatever");
        if let Err(err) = fs::remove_file(recollection_file) {
            panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
        }
    }
}
