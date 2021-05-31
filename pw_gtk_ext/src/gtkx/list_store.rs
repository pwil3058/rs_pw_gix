// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::glib;
use crate::gtkx::tree_model::*;
use crate::{are_eq_values, are_equal_as, get_row_values_from, matches_list_row, UNEXPECTED};

// NB: when done with the returned rows their items need to be unset?
#[macro_export]
macro_rules! get_rows_values_from_list {
    ( $s:expr ) => {{
        let mut rows = vec![];
        if let Some(iter) = $s.get_iter_first() {
            while $s.iter_is_valid(&iter) {
                rows.push(get_row_values_from!($s, &iter));
                $s.iter_next(&iter);
            }
        };
        rows
    }};
}

#[macro_export]
macro_rules! set_list_row_values {
    ( $s:expr, $i:expr, $r:expr ) => {{
        assert_eq!($s.get_n_columns(), $r.len() as i32);
        for (index, item) in $r.iter().enumerate() {
            $s.set_value($i, index as u32, &item);
        }
    }};
}

#[macro_export]
macro_rules! append_row_to_list {
    ( $r:expr, $s:expr ) => {{
        let iter = $s.append();
        set_list_row_values!($s, &iter, $r);
        iter
    }};
}

#[macro_export]
macro_rules! insert_row_in_list_at {
    ( $r:expr, $s:expr, $p:expr ) => {{
        let iter = $s.insert($p);
        set_list_row_values!($s, &iter, $r);
        iter
    }};
}

#[macro_export]
macro_rules! insert_row_in_list_after {
    ( $r:expr, $s:expr, $i:expr ) => {{
        let iter = $s.insert_after($i);
        set_list_row_values!($s, &iter, $r);
        iter
    }};
}

#[macro_export]
macro_rules! insert_row_in_list_before {
    ( $r:expr, $s:expr, $i:expr ) => {{
        let iter = $s.insert_before($i);
        set_list_row_values!($s, &iter, $r);
        iter
    }};
}

#[macro_export]
macro_rules! prepend_row_to_list {
    ( $r:expr, $s:expr ) => {{
        let iter = $s.prepend();
        set_list_row_values!($s, &iter, $r);
        iter
    }};
}

pub trait ListRowOps:
    TreeModelRowOps + gtk::GtkListStoreExt + gtk::prelude::GtkListStoreExtManual
{
    fn append_row(&self, row: &[glib::Value]) -> gtk::TreeIter {
        append_row_to_list!(row, self)
    }

    fn get_rows_values(&self) -> Vec<Vec<glib::Value>> {
        get_rows_values_from_list!(self)
    }

    fn insert_row(&self, position: i32, row: &[glib::Value]) -> gtk::TreeIter {
        insert_row_in_list_at!(row, self, position)
    }

    fn insert_row_after(&self, iter: &gtk::TreeIter, row: &[glib::Value]) -> gtk::TreeIter {
        insert_row_in_list_after!(row, self, Some(iter))
    }

    fn insert_row_before(&self, iter: &gtk::TreeIter, row: &[glib::Value]) -> gtk::TreeIter {
        insert_row_in_list_before!(row, self, Some(iter))
    }

    fn prepend_row(&self, row: &[glib::Value]) -> gtk::TreeIter {
        prepend_row_to_list!(row, self)
    }

    fn repopulate_with(&self, rows: &[Vec<glib::Value>]) {
        self.clear();
        for row in rows.iter() {
            append_row_to_list!(row, self);
        }
    }

    // NB: this function assumes that all rows are unique and that order isn't important
    fn update_with(&self, rows: &[Vec<glib::Value>]) {
        // First remove the rows that have gone away
        if let Some(iter) = self.get_iter_first() {
            while self.iter_is_valid(&iter) {
                let mut found = false;
                for row in rows.iter() {
                    if matches_list_row!(row, self, &iter) {
                        found = true;
                        break;
                    }
                }
                if !found {
                    // NB: this call updates the iter
                    self.remove(&iter);
                } else {
                    self.iter_next(&iter);
                }
            }
        }
        // Now add any new ones
        for row in rows.iter() {
            let position = self.find_row(row);
            if position.is_none() {
                self.append_row(row);
            }
        }
    }
}

impl ListRowOps for gtk::ListStore {}
