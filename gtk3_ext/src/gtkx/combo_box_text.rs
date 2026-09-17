// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.
use std::{cell::RefCell, error, fmt, rc::Rc};

use gtk;
use gtk::prelude::{BoxExt, ComboBoxExt, ComboBoxExtManual, ComboBoxTextExt, TreeModelExt};

use crate::wrapper::*;

#[derive(Debug)]
pub enum Error {
    DuplicateItem(String),
    UnknownItem(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DuplicateItem(item) => write!(f, "Duplicate item: {}", item),
            Error::UnknownItem(item) => write!(f, "Unknown item: {}", item),
        }
    }
}

impl error::Error for Error {}

pub trait SortedUnique {
    fn get_item_index(&self, item: &str) -> Result<i32, i32>;
    fn get_text_items(&self) -> Vec<String>;
    fn remove_text_item(&self, item: &str) -> Result<(), Error>;
    fn insert_text_item(&self, item: &str) -> Result<i32, Error>;
    fn set_active_text(&self, item: &str) -> Result<(), Error>;

    fn update_with(&self, new_item_list: &Vec<String>) {
        let current_item_list = self.get_text_items();
        for item in &current_item_list {
            if !new_item_list.contains(item) {
                self.remove_text_item(item).expect("it's there");
            }
        }
        for item in new_item_list {
            if !current_item_list.contains(item) {
                self.insert_text_item(item).expect("uniqueness checked");
            }
        }
    }
}

impl SortedUnique for gtk::ComboBoxText {
    fn get_item_index(&self, item: &str) -> Result<i32, i32> {
        if let Some(model) = self.model()
            && let Some(ref iter) = model.iter_first()
        {
            for index in 0.. {
                if let Ok(ref text) = model.value(iter, 0).get::<String>() {
                    if text == item {
                        return Ok(index);
                    } else if item < text.as_str() {
                        return Err(index);
                    }
                };
                if !model.iter_next(iter) {
                    return Err(-1);
                };
            }
        };
        Err(-1)
    }

    fn get_text_items(&self) -> Vec<String> {
        let mut text_items = Vec::new();
        if let Some(model) = self.model()
            && let Some(ref iter) = model.iter_first()
        {
            loop {
                if let Ok(ref text) = model.value(iter, 0).get::<String>() {
                    text_items.push(text.clone());
                };
                if !model.iter_next(iter) {
                    break;
                };
            }
        };
        text_items
    }

    fn remove_text_item(&self, item: &str) -> Result<(), Error> {
        match self.get_item_index(item) {
            Ok(index) => {
                self.remove(index);
                Ok(())
            }
            Err(_) => Err(Error::UnknownItem(item.to_string())),
        }
    }

    fn insert_text_item(&self, item: &str) -> Result<i32, Error> {
        match self.get_item_index(item) {
            Ok(_) => Err(Error::DuplicateItem(item.to_string())),
            Err(index) => {
                self.insert_text(index, item);
                Ok(index)
            }
        }
    }

    fn set_active_text(&self, item: &str) -> Result<(), Error> {
        match self.get_item_index(item) {
            Ok(index) => {
                self.set_active(Some(index as u32));
                Ok(())
            }
            Err(_) => Err(Error::UnknownItem(item.to_string())),
        }
    }
}

type ChangedCBs = RefCell<Vec<Box<dyn Fn(Option<String>)>>>;

#[derive(PWO)]
pub struct NameSelector {
    h_box: gtk::Box,
    combo: gtk::ComboBoxText,
    changed_callbacks: ChangedCBs,
    get_names: fn() -> Vec<String>,
}

impl NameSelector {
    pub fn new(label: &str, get_names: fn() -> Vec<String>) -> Rc<NameSelector> {
        let name_selector = Rc::new(NameSelector {
            h_box: gtk::Box::new(gtk::Orientation::Horizontal, 0),
            combo: gtk::ComboBoxText::new(),
            changed_callbacks: RefCell::new(Vec::new()),
            get_names,
        });
        let label = gtk::Label::new(Some(label)); // I18N needed here
        name_selector.h_box.pack_start(&label, false, false, 0);
        name_selector
            .h_box
            .pack_start(&name_selector.combo, true, true, 5);

        let name_selector_c = name_selector.clone();
        name_selector.combo.connect_changed(move |combo| {
            for callback in name_selector_c.changed_callbacks.borrow().iter() {
                if let Some(text) = combo.active_text() {
                    callback(Some(String::from(text)))
                } else {
                    callback(None)
                }
            }
        });

        name_selector.update_available_names();

        name_selector
    }

    pub fn get_selected_name(&self) -> Option<String> {
        self.combo.active_text().map(String::from)
    }

    pub fn set_selected_name(&self, archive_name: &str) -> Result<(), Error> {
        self.combo.set_active_text(archive_name)
    }

    pub fn update_available_names(&self) {
        let new_item_list = (self.get_names)();
        self.combo.update_with(&new_item_list);
    }

    pub fn connect_changed<F: Fn(Option<String>) + 'static>(&self, callback: F) {
        self.changed_callbacks.borrow_mut().push(Box::new(callback));
    }
}
