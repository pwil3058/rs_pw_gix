//Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::path::Path;

use gtk::{self, TreeModelExt};

pub use super::tree_model::{self, TreeModelRowOps};
pub use super::value::Row;

pub trait FileDbIfce {
    fn dir_contents(&self, dir_path: &Path, show_hidden: bool, hide_clean: bool) -> (Vec<Row>, Vec<Row>);

    fn is_current(&self) -> bool {
        true
    }

    fn reset(self) -> Self;
}

// TODO: replace this dumm FSO with proper one
pub trait FsObjectIfce {
    fn name(&self) -> String {
        "name".to_string()
    }

    fn is_dir(&self) -> bool {
        true
    }
}

impl FsObjectIfce for Row {}

pub struct FileTreeModel<FDB>
    where
        FDB: FileDbIfce,
{
    tree_store: gtk::TreeStore,
    file_db: FDB,
    show_hidden: bool,
    hide_clean: bool,
}

impl<FDB> FileTreeModel<FDB>
    where
        FDB: FileDbIfce,
{
    fn get_dir_contents(&self, dir_path: &Path) -> (Vec<Row>, Vec<Row>) {
        self.file_db.dir_contents(dir_path, self.show_hidden, self.hide_clean)
    }

    pub fn update_dir(&self, dir_path: &Path, parent_iter: Option<&gtk::TreeIter>) {
        // TODO: make sure we cater for case where dir becomes file and vice versa in a single update
        let mut _changed = false;
        let mut _place_holder_iter: Option<gtk::TreeIter> = None;
        let mut _child_iter: Option<gtk::TreeIter> = if let Some(parent_iter) = parent_iter {
            if let Some(child_iter) = self.tree_store.iter_children(parent_iter) {
                if self.tree_store.get_value(&child_iter, 0).is::<String>() {
                    //TODO: fix this condition
                    Some(child_iter)
                } else {
                    _place_holder_iter = Some(child_iter.clone());
                    if self.tree_store.iter_next(&child_iter) {
                        Some(child_iter)
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        } else {
            self.tree_store.get_iter_first()
        };
        let mut _dead_entries: Vec<gtk::TreeIter> = vec![];
        let (dirs, _files) = self.get_dir_contents(dir_path);
        for _dir_data in dirs.iter() {
            loop {
            //while (child_iter is not None) and self.get_value(child_iter, 0).is_dir and (self.get_value(child_iter, 0).name < dirdata.name):
            //    dead_entries.append(child_iter)
            //    child_iter = self.iter_next(child_iter)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
