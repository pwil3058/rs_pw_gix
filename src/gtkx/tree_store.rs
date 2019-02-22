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

use gtk;

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
