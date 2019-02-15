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

use super::list_store;

pub struct FileTreeModel {
    tree_store: gtk::TreeStore,
}

impl FileTreeModel {
    fn update_dir(&self, dir_path: &Path, parent_iter: Option<&gtk::TreeIter>) {
        // TODO: make sure we cater for case where dir becomes file and vice versa in a single update
        let mut changed = false;
        let mut place_holder_iter: Option<gtk::TreeIter> = None;
        let child_iter = if let Some(parent_iter) = parent_iter {
            if let Some(child_iter) = self.tree_store.iter_children(parent_iter) {
                if self.tree_store.get_value(&child_iter, 0).is::<String>() {
                    //TODO: fix this condition
                    Some(child_iter)
                } else {
                    place_holder_iter = Some(child_iter.clone());
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
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
