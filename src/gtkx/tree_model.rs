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

use gtk;
use gtk::prelude::*;

pub use super::value::Row;

// Macros
#[macro_export]
macro_rules! are_equal_rows {
    ( $r1:expr, $r2:expr ) => {{
        assert_eq!($r1.len(), $r2.len());
        let mut result = true;
        for i in 0..$r1.len() {
            if !are_eq_values!($r1[i], $r2[i]) {
                result = false;
                break;
            }
        }
        result
    }};
}

// NB: when done with the returned row it's items need to be unset?
#[macro_export]
macro_rules! get_row_values_from {
    ( $s:expr, $i:expr ) => {{
        let mut row = Row::new();
        let n = $s.get_n_columns();
        for index in 0..n {
            row.push($s.get_value($i, index))
        }
        row
    }};
}

// NB: when done with the returned row it's items need to be unset?
#[macro_export]
macro_rules! get_row_values_from_at {
    ( $s:expr, $p:expr ) => {{
        match $s.iter_nth_child(None, $p) {
            Some(iter) => Some(get_row_values_from!($s, &iter)),
            None => None,
        }
    }};
}

#[macro_export]
macro_rules! matches_list_row {
    ( $r:expr, $s:expr, $i:expr ) => {{
        assert_eq!($s.get_n_columns(), $r.len() as i32);
        let mut result = true;
        for (index, item) in $r.iter().enumerate() {
            let value = $s.get_value($i, index as i32);
            if !are_eq_values!(item, value) {
                result = false;
                break;
            }
        }
        result
    }};
}

#[macro_export]
macro_rules! len_of {
    ( $s:expr ) => {{
        $s.iter_n_children(None)
    }};
}

pub trait TreeModelRowOps: TreeModelExt {
    fn len(&self) -> i32 {
        len_of!(&self)
    }

    fn find_row_where<F>(&self, this_is_the_row: F) -> Option<(i32, gtk::TreeIter)>
    where
        F: Fn(&Self, &gtk::TreeIter) -> bool,
    {
        let mut index: i32 = 0;
        if let Some(iter) = self.get_iter_first() {
            loop {
                if this_is_the_row(self, &iter) {
                    return Some((index, iter));
                };
                index += 1;
                if !self.iter_next(&iter) {
                    break;
                }
            }
        };
        None
    }

    fn get_row_values(&self, iter: &gtk::TreeIter) -> Row {
        get_row_values_from!(self, iter)
    }

    fn get_row_values_at(&self, position: i32) -> Option<Row> {
        get_row_values_from_at!(self, position)
    }

    fn find_row(&self, row: &Row) -> Option<(i32, gtk::TreeIter)> {
        self.find_row_where(|list_store, iter| matches_list_row!(row, list_store, iter))
    }

    fn find_row_index(&self, row: &Row) -> Option<i32> {
        match self.find_row(row) {
            Some((index, _)) => Some(index),
            None => None,
        }
    }

    fn find_row_iter(&self, row: &Row) -> Option<gtk::TreeIter> {
        match self.find_row(row) {
            Some((_, iter)) => Some(iter),
            None => None,
        }
    }

    fn get_iter_next<'a>(&self, iter: &'a gtk::TreeIter) -> Option<&'a gtk::TreeIter> {
        if self.iter_next(&iter) {
            Some(iter)
        } else {
            None
        }
    }
}

impl TreeModelRowOps for gtk::ListStore {}
impl TreeModelRowOps for gtk::TreeStore {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn list_store_simple_row_ops() {
        if !gtk::is_initialized() {
            if let Err(err) = gtk::init() {
                panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
            };
        }

        use gtkx::list_store::ListRowOps;

        let test_list_store =
            gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String, gtk::Type::String]);
        assert_eq!(test_list_store.len(), 0);
        let row1 = vec!["one".to_value(), "two".to_value(), "three".to_value()];
        let row2 = vec!["four".to_value(), "five".to_value(), "six".to_value()];
        let row3 = vec!["seven".to_value(), "eight".to_value(), "nine".to_value()];

        test_list_store.append_row(&row1);
        assert_eq!(test_list_store.len(), 1);
        assert_eq!(test_list_store.find_row_index(&row1), Some(0));
        assert_eq!(test_list_store.find_row_index(&row2), None);
        assert_eq!(test_list_store.find_row_index(&row3), None);
        assert!(are_equal_rows!(
            test_list_store.get_row_values_at(0).unwrap(),
            row1
        ));
        assert!(test_list_store.get_row_values_at(1).is_none());

        test_list_store.prepend_row(&row2);
        assert_eq!(test_list_store.len(), 2);
        assert_eq!(test_list_store.find_row_index(&row1), Some(1));
        assert_eq!(test_list_store.find_row_index(&row2), Some(0));
        assert_eq!(test_list_store.find_row_index(&row3), None);
        assert!(are_equal_rows!(
            test_list_store.get_row_values_at(0).unwrap(),
            row2
        ));
        assert!(are_equal_rows!(
            test_list_store.get_row_values_at(1).unwrap(),
            row1
        ));
        assert!(test_list_store.get_row_values_at(2).is_none());

        test_list_store.insert_row(1, &row3);
        assert_eq!(test_list_store.len(), 3);
        assert_eq!(test_list_store.find_row_index(&row1), Some(2));
        assert_eq!(test_list_store.find_row_index(&row2), Some(0));
        assert_eq!(test_list_store.find_row_index(&row3), Some(1));
        assert!(are_equal_rows!(
            test_list_store.get_row_values_at(0).unwrap(),
            row2
        ));
        assert!(are_equal_rows!(
            test_list_store.get_row_values_at(1).unwrap(),
            row3
        ));
        assert!(are_equal_rows!(
            test_list_store.get_row_values_at(2).unwrap(),
            row1
        ));
        assert!(test_list_store.get_row_values_at(3).is_none());

        let row4 = vec!["ten".to_value(), "eleven".to_value(), "twelve".to_value()];
        let rows = vec![row1.clone(), row2.clone(), row4.clone()];
        test_list_store.update_with(&rows);
        assert_eq!(test_list_store.len(), 3);
        assert_eq!(test_list_store.find_row_index(&row1), Some(1));
        assert_eq!(test_list_store.find_row_index(&row2), Some(0));
        assert_eq!(test_list_store.find_row_index(&row3), None);
        assert_eq!(test_list_store.find_row_index(&row4), Some(2));
        assert!(are_equal_rows!(
            test_list_store.get_row_values_at(0).unwrap(),
            row2
        ));
        assert!(are_equal_rows!(
            test_list_store.get_row_values_at(1).unwrap(),
            row1
        ));
        assert!(are_equal_rows!(
            test_list_store.get_row_values_at(2).unwrap(),
            row4
        ));
        assert!(test_list_store.get_row_values_at(3).is_none());
        assert_eq!(test_list_store.get_rows_values().len(), 3);
    }

}
