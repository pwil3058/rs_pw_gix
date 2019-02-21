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

//! File system database of the current directory to feed file tree
//! stores/views

use std::cell::RefCell;
use std::clone::Clone;
use std::collections::HashMap;
use std::io::Write;
use std::iter::Iterator;
use std::rc::Rc;

//use gtk::StaticType;

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
    ) -> (Rc<Vec<DOI>>, Rc<Vec<FOI>>);

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
    path: String,
    show_hidden: bool,
    hide_clean: bool,
    dirs_data: Rc<Vec<DOI>>,
    files_data: Rc<Vec<FOI>>,
    hash_digest: Option<Vec<u8>>,
    sub_dirs: HashMap<String, OsFsDbDir<DOI, FOI>>,
}

impl<DOI, FOI> OsFsDbDir<DOI, FOI>
where
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    fn new(dir_path: &str, show_hidden: bool, hide_clean: bool) -> Self {
        OsFsDbDir::<DOI, FOI> {
            path: dir_path.to_string(),
            show_hidden: show_hidden,
            hide_clean: hide_clean,
            dirs_data: Rc::new(vec![]),
            files_data: Rc::new(vec![]),
            hash_digest: None,
            sub_dirs: HashMap::new(),
        }
    }

    fn current_hash_digest(&self) -> Vec<u8> {
        let mut hasher = Hasher::new(Algorithm::SHA256);
        if let Ok(dir_entries) = UsableDirEntry::get_entries(&self.path) {
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
        if let Ok(dir_entries) = UsableDirEntry::get_entries(&self.path) {
            let mut dirs = vec![];
            let mut files = vec![];
            for dir_entry in dir_entries {
                let path = dir_entry.path().to_string_lossy().into_owned();
                hasher.write_all(&path.into_bytes()).unwrap();
                if !self.show_hidden && dir_entry.file_name().starts_with(".") {
                    continue;
                }
                if dir_entry.is_dir() {
                    let path = dir_entry.path().to_string_lossy().into_owned();
                    dirs.push(DOI::new(&dir_entry));
                    self.sub_dirs.insert(
                        dir_entry.file_name(),
                        OsFsDbDir::<DOI, FOI>::new(&path, self.show_hidden, self.hide_clean),
                    );
                } else {
                    files.push(FOI::new(&dir_entry));
                }
            }
            self.dirs_data = Rc::new(dirs);
            self.files_data = Rc::new(files);
        }
        self.hash_digest = Some(hasher.finish());
    }

    fn find_dir(&mut self, components: &[StrPathComponent]) -> Option<&mut OsFsDbDir<DOI, FOI>> {
        if self.hash_digest.is_none() {
            self.populate();
        }
        if components.len() == 0 {
            Some(self)
        } else {
            assert!(components[0].is_normal());
            let name = components[0].to_string();
            match self.sub_dirs.get_mut(&name) {
                Some(subdir) => subdir.find_dir(&components[1..]),
                None => None,
            }
        }
    }

    fn dirs_and_files<'a>(&'a mut self) -> (Rc<Vec<DOI>>, Rc<Vec<FOI>>) {
        (Rc::clone(&self.dirs_data), Rc::clone(&self.files_data))
    }
}

pub struct OsFsDb<DOI, FOI>
where
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    base_dir: RefCell<OsFsDbDir<DOI, FOI>>,
    curr_dir: RefCell<String>, // so we can tell if there's a change of current directory
}

impl<DOI, FOI> FsDbIfce<DOI, FOI> for OsFsDb<DOI, FOI>
where
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    fn new() -> Self {
        let curr_dir = str_path_current_dir_or_panic();
        let base_dir = OsFsDbDir::<DOI, FOI>::new("", false, false); // paths are relative
        OsFsDb::<DOI, FOI> {
            base_dir: RefCell::new(base_dir),
            curr_dir: RefCell::new(curr_dir),
        }
    }

    fn dir_contents(
        &self,
        dir_path: &str,
        show_hidden: bool,
        hide_clean: bool,
    ) -> (Rc<Vec<DOI>>, Rc<Vec<FOI>>) {
        assert!(str_path_is_relative!(dir_path));
        self.check_visibility(show_hidden, hide_clean);
        let components = dir_path.to_string().path_components();
        if let Some(ref mut dir) = self.base_dir.borrow_mut().find_dir(&components) {
            dir.dirs_and_files()
        } else {
            (Rc::new(vec![]), Rc::new(vec![]))
        }
    }

    fn is_current(&self) -> bool {
        self.curr_dir_unchanged() && self.base_dir.borrow_mut().is_current()
    }

    fn reset(&mut self) {
        *self.curr_dir.borrow_mut() = str_path_current_dir_or_panic();
        *self.base_dir.borrow_mut() = OsFsDbDir::new("", false, false);
    }
}

impl<DOI, FOI> OsFsDb<DOI, FOI>
where
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    fn curr_dir_unchanged(&self) -> bool {
        *self.curr_dir.borrow() == str_path_current_dir_or_panic()
    }

    fn check_visibility(&self, show_hidden: bool, hide_clean: bool) {
        let mut base_dir = self.base_dir.borrow_mut();
        if base_dir.show_hidden != show_hidden && base_dir.hide_clean != hide_clean {
            *base_dir = OsFsDbDir::new("", show_hidden, hide_clean);
        }
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