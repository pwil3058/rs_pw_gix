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

use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use crate::fs_db::{FsDbIfce, FsObjectIfce};
pub use crate::gtkx::tree_model::TreeModelRowOps;
pub use crate::gtkx::tree_store::TreeRowOps;

trait FileTreeStoreExt: TreeRowOps {
    fn insert_place_holder(&self, dir_iter: &gtk::TreeIter) {
        self.append(dir_iter);
    }

    fn insert_place_holder_if_needed(&self, dir_iter: &gtk::TreeIter) {
        if self.iter_n_children(dir_iter) == 0 {
            self.insert_place_holder(dir_iter)
        }
    }

    fn recursive_remove(&self, iter: &gtk::TreeIter) -> bool {
        if let Some(child_iter) = self.iter_children(iter) {
            while self.recursive_remove(&child_iter) {}
        }
        self.remove(iter)
    }

    fn depopulate(&self, iter: &gtk::TreeIter) {
        if let Some(ref child_iter) = self.iter_children(iter) {
            while self.recursive_remove(child_iter) {}
        }
        self.insert_place_holder(iter)
    }

    fn remove_dead_rows<'a, F: Fn(&Self, &gtk::TreeIter) -> bool>(
        &self,
        mut o_iter: Option<&'a gtk::TreeIter>,
        until: F,
        changed: &mut bool,
    ) -> Option<&'a gtk::TreeIter> {
        loop {
            if let Some(iter) = o_iter {
                if until(self, iter) {
                    break;
                }
                o_iter = if self.recursive_remove(iter) {
                    Some(iter)
                } else {
                    None
                };
                *changed = true;
            } else {
                break;
            }
        }
        o_iter
    }
}

impl FileTreeStoreExt for gtk::TreeStore {}

trait FileTreeIfce<FSDB, DOI, FOI>
where
    FSDB: FsDbIfce<DOI, FOI>,
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    fn new(auto_expand: bool) -> Self;
    fn view(&self) -> &gtk::TreeView;
    fn store(&self) -> &gtk::TreeStore;
    fn fs_db(&self) -> &FSDB;
    fn auto_expand(&self) -> bool;
    fn show_hidden(&self) -> bool;
    fn hide_clean(&self) -> bool;

    fn get_dir_contents(&self, dir_path: &str) -> (Rc<Vec<DOI>>, Rc<Vec<FOI>>) {
        self.fs_db()
            .dir_contents(dir_path, self.show_hidden(), self.hide_clean())
    }

    fn view_expand_row(&self, dir_iter: &gtk::TreeIter) {
        if let Some(ref path) = self.store().get_path(&dir_iter) {
            self.view().expand_row(path, true);
        }
    }

    fn view_row_expanded(&self, iter: &gtk::TreeIter) -> bool {
        if let Some(ref path) = self.store().get_path(&iter) {
            return self.view().row_expanded(path);
        }
        false
    }

    fn auto_expand_dir_or_insert_place_holder(&self, dir_path: &str, dir_iter: &gtk::TreeIter) {
        if self.auto_expand() {
            self.populate_dir(dir_path, Some(dir_iter));
            self.view_expand_row(dir_iter);
        } else {
            self.store().insert_place_holder(dir_iter);
        }
    }

    fn populate_dir(&self, dir_path: &str, o_parent_iter: Option<&gtk::TreeIter>) {
        let (dirs, files) = self.get_dir_contents(dir_path);
        for dir_data in dirs.iter() {
            let dir_iter = self.store().append(o_parent_iter);
            dir_data.set_row_values(self.store(), &dir_iter);
            self.auto_expand_dir_or_insert_place_holder(dir_path, &dir_iter);
        }
        for file_data in files.iter() {
            let file_iter = self.store().append(o_parent_iter);
            file_data.set_row_values(self.store(), &file_iter);
        }
        if let Some(parent_iter) = o_parent_iter {
            self.store().insert_place_holder_if_needed(parent_iter)
        }
    }

    fn repopulate(&mut self) {
        // TODO: add show_busy() mechanism here
        self.fs_db().reset();
        self.store().clear();
        if let Some(iter) = self.store().get_iter_first() {
            self.populate_dir("", Some(&iter))
        } else {
            self.populate_dir("", None)
        }
    }

    fn update_dir(&self, dir_path: &str, o_parent_iter: Option<&gtk::TreeIter>) -> bool {
        // TODO: make sure we cater for case where dir becomes file and vice versa in a single update
        let mut changed = false;
        let mut o_place_holder_iter: Option<gtk::TreeIter> = None;
        let child_iter: gtk::TreeIter; // needed to satisfy lifetimes
        let mut o_child_iter: Option<&gtk::TreeIter> = if let Some(parent_iter) = o_parent_iter {
            if let Some(iter) = self.store().iter_children(parent_iter) {
                child_iter = iter;
                if self.store().get_value(&child_iter, 0).is::<String>() {
                    //TODO: fix this condition
                    Some(&child_iter)
                } else {
                    o_place_holder_iter = Some(child_iter.clone());
                    self.store().get_iter_next(&child_iter)
                }
            } else {
                None
            }
        } else if let Some(iter) = self.store().get_iter_first() {
            child_iter = iter.clone();
            Some(&child_iter)
        } else {
            None
        };
        let (dirs, files) = self.get_dir_contents(dir_path);
        for dir_data in dirs.iter() {
            o_child_iter = self.store().remove_dead_rows(
                o_child_iter,
                |s, i| {
                    !FOI::row_is_a_dir(s, i)
                        || FOI::get_name_from_row(s, i).as_str() >= dir_data.name()
                },
                &mut changed,
            );
            if let Some(child_iter) = o_child_iter {
                let name = FOI::get_name_from_row(self.store(), &child_iter);
                if !FOI::row_is_a_dir(self.store(), &child_iter) || name.as_str() > dir_data.name()
                {
                    let dir_iter = self.store().insert_before(o_parent_iter, o_child_iter);
                    dir_data.set_row_values(self.store(), &dir_iter);
                    changed = true;
                    self.auto_expand_dir_or_insert_place_holder(&dir_data.path(), &dir_iter);
                } else {
                    changed |= dir_data.update_row_if_required(self.store(), child_iter);
                    // This is an update so ignore auto_expand for existing directories
                    // BUT update them if they"re already expanded
                    if self.view_row_expanded(child_iter) {
                        changed |= self.update_dir(dir_data.path(), o_child_iter);
                    } else {
                        // make sure we don"t leave bad data in children that were previously expanded
                        self.store().depopulate(child_iter);
                    }
                    o_child_iter = self.store().get_iter_next(child_iter);
                }
            } else {
                let dir_iter = self.store().append(o_parent_iter);
                dir_data.set_row_values(self.store(), &dir_iter);
                changed = true;
                self.auto_expand_dir_or_insert_place_holder(&dir_data.path(), &dir_iter);
            }
        }
        o_child_iter = self.store().remove_dead_rows(
            o_child_iter,
            |s, i| !FOI::row_is_a_dir(s, i),
            &mut changed,
        );
        for file_data in files.iter() {
            o_child_iter = self.store().remove_dead_rows(
                o_child_iter,
                |s, i| FOI::get_name_from_row(s, i).as_str() >= file_data.name(),
                &mut changed,
            );
            if let Some(child_iter) = o_child_iter {
                if FOI::get_name_from_row(self.store(), child_iter).as_str() > file_data.name() {
                    changed = true;
                    let file_iter = self.store().insert_before(o_parent_iter, o_child_iter);
                    file_data.set_row_values(self.store(), &file_iter);
                } else {
                    changed |= file_data.update_row_if_required(self.store(), child_iter);
                    o_child_iter = self.store().get_iter_next(child_iter);
                }
            } else {
                let file_iter = self.store().append(o_parent_iter);
                file_data.set_row_values(self.store(), &file_iter);
                changed = true;
            }
        }
        self.store()
            .remove_dead_rows(o_child_iter, |_, _| false, &mut changed);

        if let Some(parent_iter) = o_parent_iter {
            let n_children = self.store().iter_n_children(parent_iter);
            if n_children == 0 {
                self.store().insert_place_holder(parent_iter)
            } else if n_children > 1 {
                if let Some(place_holder_iter) = o_place_holder_iter {
                    //assert_eq!(self.get_value(iter, 0), None);
                    self.store().remove(&place_holder_iter);
                }
            }
        }
        changed
    }

    fn update(&mut self) -> bool {
        // TODO: add show_busy() mechanism here
        if self.fs_db().is_current() {
            false
        } else {
            self.fs_db().reset();
            self.update_dir("", None)
        }
    }
}
