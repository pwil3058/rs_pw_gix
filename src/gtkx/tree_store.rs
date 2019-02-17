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

use gtk::{self, TreeModelExt, TreeStoreExt, TreeViewExt};

pub use super::tree_model::{self, TreeModelRowOps};
pub use super::value::Row;

#[macro_export]
macro_rules! set_tree_row_values {
    ( $s:expr, $i:expr, $r:expr ) => {{
        assert_eq!($s.get_n_columns(), $r.len() as i32);
        for (index, item) in $r.iter().enumerate() {
            $s.set_value($i, index as u32, &item);
        }
    }};
}

#[macro_export]
macro_rules! append_row_to_tree {
    ( $r:expr, $s:expr, $i:expr ) => {{
        let iter = $s.append($i);
        set_tree_row_values!($s, &iter, $r);
        iter
    }};
}

#[macro_export]
macro_rules! insert_row_in_tree_at {
    ( $r:expr, $s:expr, $p:expr, $i:expr ) => {{
        let iter = $s.insert($p, $i);
        set_list_row_values!($s, &iter, $r);
        iter
    }};
}

#[macro_export]
macro_rules! insert_row_in_tree_after {
    ( $r:expr, $s:expr, $p:expr, $q:expr ) => {{
        let iter = $s.insert_after($p, $q);
        set_list_row_values!($s, &iter, $r);
        iter
    }};
}

#[macro_export]
macro_rules! insert_row_in_tree_before {
    ( $r:expr, $s:expr, $p:expr, $q:expr ) => {{
        let iter = $s.insert_before($p, $q);
        set_list_row_values!($s, &iter, $r);
        iter
    }};
}

#[macro_export]
macro_rules! prepend_row_to_tree {
    ( $r:expr, $s:expr, $i:expr ) => {{
        let iter = $s.prepend($i);
        set_tree_row_values!($s, &iter, $r);
        iter
    }};
}

pub trait TreeRowOps:
    TreeModelRowOps + gtk::TreeStoreExt + gtk::prelude::TreeStoreExtManual
{
    fn append_row<'a, P>(&self, row: &Row, parent: P) -> gtk::TreeIter
    where
        P: Into<Option<&'a gtk::TreeIter>>,
    {
        append_row_to_tree!(row, self, parent)
    }

    fn insert_row<'a, P>(&self, row: &Row, parent: P, position: i32) -> gtk::TreeIter
    where
        P: Into<Option<&'a gtk::TreeIter>>,
    {
        insert_row_in_tree_at!(row, self, parent, position)
    }

    fn insert_row_after<'a, 'b, P, Q>(&self, row: &Row, parent: P, sibling: Q) -> gtk::TreeIter
    where
        P: Into<Option<&'a gtk::TreeIter>>,
        Q: Into<Option<&'b gtk::TreeIter>>,
    {
        insert_row_in_tree_after!(row, self, parent, sibling)
    }

    fn insert_row_before<'a, 'b, P, Q>(&self, row: &Row, parent: P, sibling: Q) -> gtk::TreeIter
    where
        P: Into<Option<&'a gtk::TreeIter>>,
        Q: Into<Option<&'b gtk::TreeIter>>,
    {
        insert_row_in_tree_before!(row, self, parent, sibling)
    }

    fn prepend_row<'a, P>(&self, row: &Row, parent: P) -> gtk::TreeIter
    where
        P: Into<Option<&'a gtk::TreeIter>>,
    {
        prepend_row_to_tree!(row, self, parent)
    }

    fn set_row_values(&self, row: &Row, iter: &gtk::TreeIter) {
        set_tree_row_values!(self, iter, row);
    }
}

impl TreeRowOps for gtk::TreeStore {}

// File Tree Store
use std::marker::PhantomData;
use crate::fs_db::{FsDbIfce, FsObjectIfce};

pub struct FileTreeStore<FDB, DOI, FOI>
where
    FDB: FsDbIfce<DOI, FOI>,
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    tree_store: gtk::TreeStore,
    file_db: FDB,
    show_hidden: bool,
    hide_clean: bool,
    auto_expand: bool,
    o_view: Option<gtk::TreeView>,
    phantom: PhantomData<(DOI, FOI)>,
}

impl<FDB, DOI, FOI> FileTreeStore<FDB, DOI, FOI>
where
    FDB: FsDbIfce<DOI, FOI>,
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    pub fn new(o_view: Option<gtk::TreeView>, auto_expand: bool) -> Self {
        let fspec: Vec<gtk::Type> = FOI::tree_store_spec();
        let dspec: Vec<gtk::Type> = DOI::tree_store_spec();
        assert_eq!(fspec, dspec);

        FileTreeStore::<FDB, DOI, FOI> {
            tree_store: gtk::TreeStore::new(&fspec),
            file_db: FDB::new(),
            show_hidden: false,
            hide_clean: false,
            auto_expand: auto_expand,
            o_view: o_view,
            phantom: PhantomData,
        }
    }

    fn get_dir_contents(&self, dir_path: &str) -> (Vec<DOI>, Vec<FOI>) {
        self.file_db
            .dir_contents(dir_path, self.show_hidden, self.hide_clean)
    }

    fn view_expand_row(&self, dir_iter: &gtk::TreeIter) {
        if let Some(ref view) = self.o_view {
            if let Some(ref path) = self.tree_store.get_path(&dir_iter) {
                view.expand_row(path, true);
            }
        }
    }

    fn view_row_expanded(&self, iter: &gtk::TreeIter) -> bool {
        if let Some(ref view) = self.o_view {
            if let Some(ref path) = self.tree_store.get_path(&iter) {
                return view.row_expanded(path);
            }
        }
        false
    }

    fn insert_place_holder(&self, dir_iter: &gtk::TreeIter) {
        self.tree_store.append(dir_iter);
    }

    fn insert_place_holder_if_needed(&self, dir_iter: &gtk::TreeIter) {
        if self.tree_store.iter_n_children(dir_iter) == 0 {
            self.insert_place_holder(dir_iter)
        }
    }

    fn recursive_remove(&self, iter: &gtk::TreeIter) -> bool {
        if let Some(child_iter) = self.tree_store.iter_children(iter) {
            while self.recursive_remove(&child_iter) {}
        }
        self.tree_store.remove(iter)
    }

    fn depopulate(&self, iter: &gtk::TreeIter) {
        if let Some(ref child_iter) = self.tree_store.iter_children(iter) {
            while self.recursive_remove(child_iter) {}
        }
        self.insert_place_holder(iter)
    }

    fn remove_dead_rows<'a, F: Fn(&Row) -> bool>(
        &self,
        mut o_iter: Option<&'a gtk::TreeIter>,
        until: F,
        changed: &mut bool,
    ) -> Option<&'a gtk::TreeIter> {
        loop {
            if let Some(iter) = o_iter {
                let values = self.tree_store.get_row_values(iter);
                if until(&values) {
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

    fn populate_dir(&self, dir_path: &str, o_parent_iter: Option<&gtk::TreeIter>) {
        let (dirs, files) = self.get_dir_contents(dir_path);
        for dir_data in dirs.iter() {
            let dir_iter = self.tree_store.append_row(&dir_data.row(), o_parent_iter);
            if self.auto_expand {
                // TODO: make auto_expand function to handle this
                self.populate_dir(dir_data.path(), Some(&dir_iter));
                self.view_expand_row(&dir_iter);
            } else {
                self.insert_place_holder(&dir_iter);
            }
        }
        for file_data in files.iter() {
            self.tree_store.append_row(&file_data.row(), o_parent_iter);
        }
        if let Some(parent_iter) = o_parent_iter {
            self.insert_place_holder_if_needed(parent_iter)
        }
    }

    pub fn repopulate(&mut self) {
        // TODO: add show_busy() mechanism here
        self.file_db = FDB::new();
        self.tree_store.clear();
        if let Some(iter) = self.tree_store.get_iter_first() {
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
            if let Some(iter) = self.tree_store.iter_children(parent_iter) {
                child_iter = iter;
                if self.tree_store.get_value(&child_iter, 0).is::<String>() {
                    //TODO: fix this condition
                    Some(&child_iter)
                } else {
                    o_place_holder_iter = Some(child_iter.clone());
                    self.tree_store.get_iter_next(&child_iter)
                }
            } else {
                None
            }
        } else if let Some(iter) = self.tree_store.get_iter_first() {
            child_iter = iter.clone();
            Some(&child_iter)
        } else {
            None
        };
        let (dirs, files) = self.get_dir_contents(dir_path);
        for dir_data in dirs.iter() {
            o_child_iter = self.remove_dead_rows(
                o_child_iter,
                |r| !FOI::row_is_a_dir(r) || FOI::get_name_from_row(r) >= dir_data.name(),
                &mut changed,
            );
            if let Some(child_iter) = o_child_iter {
                let values = self.tree_store.get_row_values(&child_iter);
                let name = FOI::get_name_from_row(&values);
                if !FOI::row_is_a_dir(&values) || name > dir_data.name() {
                    let dir_iter = self.tree_store.insert_row_before(
                        dir_data.row(),
                        o_parent_iter,
                        o_child_iter,
                    );
                    changed = true;
                    if self.auto_expand {
                        self.populate_dir(dir_data.path(), Some(&dir_iter));
                        self.view_expand_row(&dir_iter);
                    } else {
                        self.insert_place_holder(&dir_iter);
                    }
                    continue;
                }
                if !dir_data.row_is_the_same(&values) {
                    changed = true;
                    self.tree_store
                        .set_row_values(dir_data.row(), o_child_iter.unwrap());
                }
                // This is an update so ignore auto_expand for existing directories
                // BUT update them if they"re already expanded
                if self.view_row_expanded(child_iter) {
                    changed |= self.update_dir(dir_data.path(), o_child_iter);
                } else {
                    // make sure we don"t leave bad data in children that were previously expanded
                    self.depopulate(child_iter);
                }
                o_child_iter = self.tree_store.get_iter_next(child_iter);
            } else {
                let dir_iter = self.tree_store.append_row(dir_data.row(), o_parent_iter);
                changed = true;
                if self.auto_expand {
                    self.populate_dir(dir_data.path(), Some(&dir_iter));
                    self.view_expand_row(&dir_iter);
                } else {
                    self.insert_place_holder(&dir_iter);
                }
            }
        }
        o_child_iter = self.remove_dead_rows(o_child_iter, |r| !FOI::row_is_a_dir(r), &mut changed);
        for file_data in files {
            o_child_iter = self.remove_dead_rows(
                o_child_iter,
                |r| FOI::get_name_from_row(r) >= file_data.name(),
                &mut changed,
            );
            if let Some(child_iter) = o_child_iter {
                let values = self.tree_store.get_row_values(&child_iter);
                if FOI::get_name_from_row(&values) > file_data.name() {
                    changed = true;
                    self.tree_store
                        .insert_row_before(file_data.row(), o_parent_iter, o_child_iter);
                } else if !file_data.row_is_the_same(&values) {
                    changed = true;
                    self.tree_store
                        .set_row_values(file_data.row(), o_child_iter.unwrap());
                    o_child_iter = self.tree_store.get_iter_next(child_iter);
                } else {
                    o_child_iter = self.tree_store.get_iter_next(child_iter);
                }
            } else {
                self.tree_store.append_row(file_data.row(), o_parent_iter);
                changed = true;
            }
        }
        self.remove_dead_rows(o_child_iter, |_| false, &mut changed);

        if let Some(parent_iter) = o_parent_iter {
            let n_children = self.tree_store.iter_n_children(parent_iter);
            if n_children == 0 {
                self.insert_place_holder(parent_iter)
            } else if n_children > 1 {
                if let Some(place_holder_iter) = o_place_holder_iter {
                    //assert_eq!(self.get_value(iter, 0), None);
                    self.tree_store.remove(&place_holder_iter);
                }
            }
        }
        changed
    }

    pub fn update(&mut self, fsdb_reset_only: bool) -> bool {
        // TODO: add show_busy() mechanism here
        if fsdb_reset_only {
            self.file_db.reset();
        } else {
            self.file_db = FDB::new();
        };
        self.update_dir("", None)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
