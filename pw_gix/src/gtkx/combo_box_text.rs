// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use gtk;
use gtk::prelude::{ComboBoxExt, ComboBoxExtManual, ComboBoxTextExt, TreeModelExt};

pub trait SortedUnique {
    fn get_item_index(&self, item: &str) -> (bool, i32);
    fn get_text_items(&self) -> Vec<String>;
    fn remove_text_item(&self, item: &str) -> bool;
    fn insert_text_item(&self, item: &str) -> i32;
    fn set_active_text(&self, item: &str);

    fn update_with(&self, new_item_list: &Vec<String>) {
        let current_item_list = self.get_text_items();
        for item in &current_item_list {
            if !new_item_list.contains(&item) {
                self.remove_text_item(&item);
            }
        }
        for item in new_item_list {
            if !current_item_list.contains(&item) {
                self.insert_text_item(&item);
            }
        }
    }
}

impl SortedUnique for gtk::ComboBoxText {
    fn get_item_index(&self, item: &str) -> (bool, i32) {
        if let Some(model) = self.get_model() {
            if let Some(ref iter) = model.get_iter_first() {
                for index in 0.. {
                    if let Some(ref text) = model.get_value(iter, 0).get::<String>() {
                        if text == item {
                            return (true, index);
                        } else if item < text.as_str() {
                            return (false, index);
                        }
                    };
                    if !model.iter_next(iter) {
                        return (false, -1);
                    };
                }
            }
        };
        return (false, -1);
    }

    fn get_text_items(&self) -> Vec<String> {
        let mut text_items = Vec::new();
        if let Some(model) = self.get_model() {
            if let Some(ref iter) = model.get_iter_first() {
                loop {
                    if let Some(ref text) = model.get_value(iter, 0).get::<String>() {
                        text_items.push(text.clone());
                    };
                    if !model.iter_next(iter) {
                        break;
                    };
                }
            }
        };
        text_items
    }

    fn remove_text_item(&self, item: &str) -> bool {
        let (found, index) = self.get_item_index(item);
        if found {
            self.remove(index);
        };
        found
    }

    fn insert_text_item(&self, item: &str) -> i32 {
        let (found, index) = self.get_item_index(item);
        if !found {
            self.insert_text(index, item);
        } else {
            panic!(
                "{:?}: line {:?}: {}: items must be unique",
                file!(),
                line!(),
                item
            )
        };
        index
    }

    fn set_active_text(&self, item: &str) {
        let (found, index) = self.get_item_index(item);
        if found {
            self.set_active(Some(index as u32));
        } else {
            panic!("{:?}: line {:?}: {}: unknown item", file!(), line!(), item)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn combo_box_text_sorted_unique() {
        init_gtk_if_needed!();

        let cbt = gtk::ComboBoxText::new();
        assert!(!cbt.remove_text_item("one"));
        assert_eq!(cbt.insert_text_item("one"), -1);
        assert_eq!(cbt.insert_text_item("two"), -1);
        assert_eq!(cbt.insert_text_item("three"), 1);
        assert_eq!(cbt.insert_text_item("four"), 0);
        assert_eq!(cbt.insert_text_item("five"), 0);
        assert_eq!(cbt.insert_text_item("six"), 3);
        assert_eq!(cbt.insert_text_item("zero"), -1);
        assert!(cbt.remove_text_item("two"));
        assert!(!cbt.remove_text_item("two"));
        assert!(cbt.remove_text_item("four"));
        assert!(!cbt.remove_text_item("four"));
        assert_eq!(
            cbt.get_text_items(),
            vec!["five", "one", "six", "three", "zero"]
        );
        assert_ne!(
            cbt.get_text_items(),
            vec!["five", "one", "six", "ten", "three", "zero"]
        );
        cbt.update_with(&vec![
            "five".to_string(),
            "one".to_string(),
            "ten".to_string(),
            "three".to_string(),
            "zero".to_string(),
            "twelve".to_string(),
            "aa".to_string(),
            "zz".to_string(),
        ]);
        assert_eq!(
            cbt.get_text_items(),
            vec!["aa", "five", "one", "ten", "three", "twelve", "zero", "zz"]
        );
    }
}
