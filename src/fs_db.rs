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

//! File system database to feed file tree stores/views

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{ErrorKind, Write};

use gtk::StaticType;

use crypto_hash::{Algorithm, Hasher};

use pw_pathux::str_path::*;
use pw_pathux::UsableDirEntry;

pub use crate::gtkx::value::Row;

pub trait FsObjectIfce {
    fn new(dir_entry: &UsableDirEntry) -> Self;
    fn tree_store_spec() -> Vec<gtk::Type>;
    fn row_is_a_dir(row: &Row) -> bool;
    fn get_name_from_row(row: &Row) -> &str;
    fn get_path_from_row(row: &Row) -> &str;

    fn row_is_the_same(&self, row: &Row) -> bool;
    fn name(&self) -> &str;
    fn path(&self) -> &str;
    fn is_dir(&self) -> bool;
    fn row(&self) -> &Row;
}

pub trait FsDbIfce<DOI, FOI>
where
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    fn new() -> Self;

    fn dir_contents(
        &self,
        dir_path: &str,
        show_hidden: bool,
        hide_clean: bool,
    ) -> (Vec<DOI>, Vec<FOI>);

    fn is_current(&self) -> bool {
        true
    }

    fn reset(&mut self);
}

// Plain OS FS Database

pub struct OsFsDbDir<DOI, FOI>
where
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    data: DOI,
    dirs_data: Vec<DOI>,
    files_data: Vec<FOI>,
    hash_digest: Option<Vec<u8>>,
    sub_dirs: HashMap<String, OsFsDbDir<DOI, FOI>>,
}

impl<DOI, FOI> OsFsDbDir<DOI, FOI>
where
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    fn new(dir_entry: &UsableDirEntry) -> Self {
        OsFsDbDir::<DOI, FOI> {
            data: DOI::new(dir_entry),
            dirs_data: vec![],
            files_data: vec![],
            hash_digest: None,
            sub_dirs: HashMap::new(),
        }
    }

    fn current_hash_digest(&self) -> Vec<u8> {
        let mut hasher = Hasher::new(Algorithm::SHA256);
        if let Ok(dir_entries) = UsableDirEntry::get_entries(&self.data.path()) {
            for dir_entry in dir_entries {
                let path = dir_entry.path().to_string_lossy().into_owned();
                hasher.write_all(&path.into_bytes()).unwrap()
            }
        }
        hasher.finish()
    }

    fn is_current(&self) -> bool {
        match self.hash_digest {
            None => return true,
            Some(ref hash_digest) => {
                if *hash_digest != self.current_hash_digest() {
                    return false;
                } else {
                    for sub_dir in self.sub_dirs.values() {
                        if !sub_dir.is_current() {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    fn populate(&mut self) {
        let mut hasher = Hasher::new(Algorithm::SHA256);
        if let Ok(dir_entries) = UsableDirEntry::get_entries(&self.data.path()) {
            for dir_entry in dir_entries {
                let path = dir_entry.path().to_string_lossy().into_owned();
                hasher.write_all(&path.into_bytes()).unwrap();
                if dir_entry.is_dir() {
                    self.dirs_data.push(DOI::new(&dir_entry));
                    self.sub_dirs.insert(
                        dir_entry.file_name(),
                        OsFsDbDir::<DOI, FOI>::new(&dir_entry),
                    );
                } else {
                    self.files_data.push(FOI::new(&dir_entry));
                }
            }
        }
        self.hash_digest = Some(hasher.finish());
    }
}

//lazy_static! {
//    pub static ref OS_FS_DB_ROW_SPEC: [gtk::Type; 5] =
//        [
//            gtk::Type::String,          // 0 Path
//            gtk::Type::String,          // 1 Status
//            gtk::Type::String,          // 2 Related file data
//            gtk::Type::String,          // 3 icon
//            bool::static_type(),        // 4 is a directory?
//        ];
//}

//pub struct OsFileData {
//    path: String,
//    status: String,
//    related_file_data: String,
//    icon: String,
//}
