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

pub trait FsObjectIfce {
    fn row_is_a_dir(row: &Row) -> bool;
    fn get_name_from_row(row: &Row) -> &str;

    fn row_is_the_same(&self, row: &Row) -> bool;
    fn name(&self) -> &str;
    fn is_dir(&self) -> bool;
    fn row(&self) -> &Row;
}

pub trait FileDbIfce<FOI>
where
    FOI: FsObjectIfce,
{
    fn dir_contents(
        &self,
        dir_path: &Path,
        show_hidden: bool,
        hide_clean: bool,
    ) -> (Vec<FOI>, Vec<FOI>);

    fn is_current(&self) -> bool {
        true
    }

    fn reset(self) -> Self;
}

pub struct FileTreeModel<FDB, FOI>
where
    FDB: FileDbIfce<FOI>,
    FOI: FsObjectIfce,
{
    tree_store: gtk::TreeStore,
    file_db: FDB,
    show_hidden: bool,
    hide_clean: bool,
    auto_expand: bool,
    view: Option<gtk::TreeView>,
    phantom: PhantomData<FOI>,
}

impl<FDB, FOI> FileTreeModel<FDB, FOI>
where
    FDB: FileDbIfce<FOI>,
    FOI: FsObjectIfce,
{
    fn get_dir_contents(&self, dir_path: &Path) -> (Vec<FOI>, Vec<FOI>) {
        self.file_db
            .dir_contents(dir_path, self.show_hidden, self.hide_clean)
    }

    fn view_expand_row(&self, dir_iter: &gtk::TreeIter) {
        if let Some(ref view) = self.view {
            if let Some(ref path) = self.tree_store.get_path(&dir_iter) {
                view.expand_row(path, true);
            }
        }
    }

    fn view_row_expanded(&self, iter: &gtk::TreeIter) -> bool {
        if let Some(ref view) = self.view {
            if let Some(ref path) = self.tree_store.get_path(&iter) {
                return view.row_expanded(path);
            }
        }
        false
    }

    fn insert_place_holder(&self, dir_iter: &gtk::TreeIter) {
        self.tree_store.append(dir_iter);
    }

    fn _recursive_remove(self, iter: &gtk::TreeIter) -> bool {
        //        if let Some(ref child_iter) = self.tree_store.iter_children(iter) {
        //            while self.recursive_remove(child_iter) {}
        //        }
        return self.tree_store.remove(iter);
    }

    fn depopulate(&self, _iter: &gtk::TreeIter) {
        //        if let Some(ref child_iter) = self.tree_store.iter_children(iter) {
        //            while self.recursive_remove(child_iter) {}
        //        }
        //        self.insert_place_holder(iter)
    }

    pub fn update_dir(&self, dir_path: &Path, parent_iter: Option<&gtk::TreeIter>) -> bool {
        // TODO: make sure we cater for case where dir becomes file and vice versa in a single update
        let mut changed = false;
        let mut _place_holder_iter: Option<gtk::TreeIter> = None;
        let long_lived_iter: gtk::TreeIter;
        let mut o_child_iter: Option<&gtk::TreeIter> = if let Some(parent_iter) = parent_iter {
            if let Some(iter) = self.tree_store.iter_children(parent_iter) {
                if self.tree_store.get_value(&iter, 0).is::<String>() {
                    //TODO: fix this condition
                    long_lived_iter = iter.clone();
                    Some(&long_lived_iter)
                } else {
                    _place_holder_iter = Some(iter.clone());
                    if self.tree_store.iter_next(&iter) {
                        long_lived_iter = iter.clone();
                        Some(&long_lived_iter)
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        } else if let Some(iter) = self.tree_store.get_iter_first() {
            long_lived_iter = iter.clone();
            Some(&long_lived_iter)
        } else {
            None
        };
        let mut dead_entries: Vec<gtk::TreeIter> = vec![];
        let (dirs, _files) = self.get_dir_contents(dir_path);
        for dir_data in dirs.iter() {
            loop {
                if let Some(ci) = o_child_iter.clone() {
                    let values = self.tree_store.get_row_values(&ci);
                    if !FOI::row_is_a_dir(&values)
                        || FOI::get_name_from_row(&values) < dir_data.name()
                    {
                        break;
                    }
                    dead_entries.push(ci.clone());
                    o_child_iter = self.tree_store.get_iter_next(&ci);
                } else {
                    break;
                }
            }
            if o_child_iter.is_none() {
                let dir_iter = self.tree_store.append_row(dir_data.row(), parent_iter);
                changed = true;
                if self.auto_expand {
                    self.update_dir(&dir_path.join(dir_data.name()), Some(&dir_iter));
                    self.view_expand_row(&dir_iter);
                } else {
                    self.insert_place_holder(&dir_iter);
                }
                continue;
            }
            let child_iter = o_child_iter.clone().unwrap();
            let values = self.tree_store.get_row_values(&child_iter);
            let name = FOI::get_name_from_row(&values);
            if !FOI::row_is_a_dir(&values) || name > dir_data.name() {
                let dir_iter =
                    self.tree_store
                        .insert_row_before(dir_data.row(), parent_iter, o_child_iter);
                changed = true;
                if self.auto_expand {
                    self.update_dir(&dir_path.join(dir_data.name()), Some(&dir_iter));
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
                changed |= self.update_dir(&dir_path.join(name), o_child_iter);
            } else {
                // make sure we don"t leave bad data in children that were previously expanded
                self.depopulate(child_iter);
            }
            o_child_iter = self.tree_store.get_iter_next(child_iter);
        }
        changed |= dead_entries.len() > 0;
        changed
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
