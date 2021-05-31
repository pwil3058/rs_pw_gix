// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::glib::Value;
use crate::gtkx::buffered_list_store::{BufferedListStore, RawDataSource};
use crate::gtkx::menu::{ManagedMenu, ManagedMenuBuilder, MenuItemSpec};
use crate::sav_state::MaskedCondns;
use crate::sourceview::prelude::{
    GtkMenuItemExt, TreeModelExt, TreeSelectionExt, TreeViewExt, WidgetExt,
};
use crate::wrapper::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type PopupCallback = Box<dyn Fn(Option<Value>, Option<Vec<Value>>)>;

#[derive(PWO)]
pub struct BufferedListViewCore<R: RawDataSource> {
    view: gtk::TreeView,
    list_store: BufferedListStore<R>,
    selected_id: RefCell<Option<Value>>,
    popup_menu: ManagedMenu,
    callbacks: RefCell<HashMap<String, Vec<PopupCallback>>>,
    id_field: i32,
}

#[derive(PWO, WClone)]
pub struct BufferedListView<R: RawDataSource>(Rc<BufferedListViewCore<R>>);

impl<R: RawDataSource> BufferedListView<R> {
    fn set_selected_id(&self, posn: (f64, f64)) {
        if let Some(location) = self.0.view.get_path_at_pos(posn.0 as i32, posn.1 as i32) {
            if let Some(path) = location.0 {
                if let Some(list_store) = self.0.view.get_model() {
                    if let Some(iter) = list_store.get_iter(&path) {
                        let value = list_store.get_value(&iter, self.0.id_field);
                        *self.0.selected_id.borrow_mut() = Some(value);
                        self.0.popup_menu.update_hover_condns(true);
                        return;
                    }
                }
            }
        };
        *self.0.selected_id.borrow_mut() = None;
        self.0.popup_menu.update_hover_condns(false);
    }

    pub fn connect_popup_menu_item<F: Fn(Option<Value>, Option<Vec<Value>>) + 'static>(
        &self,
        name: &str,
        callback: F,
    ) {
        self.0
            .callbacks
            .borrow_mut()
            .get_mut(name)
            .expect("invalid name")
            .push(Box::new(callback));
    }

    fn menu_item_selected(&self, name: &str) {
        let hovered_id = if let Some(ref id) = *self.0.selected_id.borrow() {
            Some(id.clone())
        } else {
            None
        };
        let selection = self.0.view.get_selection();
        let (tree_paths, store) = selection.get_selected_rows();
        let selected_ids: Option<Vec<Value>> = if tree_paths.len() > 0 {
            let mut vector = vec![];
            for tree_path in tree_paths.iter() {
                if let Some(iter) = store.get_iter(&tree_path) {
                    vector.push(store.get_value(&iter, self.0.id_field));
                }
            }
            if vector.is_empty() {
                None
            } else {
                Some(vector)
            }
        } else {
            None
        };
        if hovered_id.is_some() || selected_ids.is_some() {
            for callback in self
                .0
                .callbacks
                .borrow()
                .get(name)
                .expect("invalid name")
                .iter()
            {
                callback(hovered_id.clone(), selected_ids.clone())
            }
        }
    }

    pub fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        self.0.popup_menu.update_condns(changed_condns)
    }

    pub fn repopulate(&self) {
        self.0.list_store.repopulate()
    }

    pub fn update(&self) {
        self.0.list_store.update()
    }
}

pub struct BufferedListViewBuilder {
    menu_items: Vec<(&'static str, MenuItemSpec, u64)>,
    id_field: i32,
    selection_mode: gtk::SelectionMode,
}

impl Default for BufferedListViewBuilder {
    fn default() -> Self {
        Self {
            menu_items: vec![],
            id_field: 0,
            selection_mode: gtk::SelectionMode::Single,
        }
    }
}

impl BufferedListViewBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn menu_item(&mut self, menu_item: (&'static str, MenuItemSpec, u64)) -> &mut Self {
        self.menu_items.push(menu_item);
        self
    }

    pub fn menu_items(&mut self, menu_items: Vec<(&'static str, MenuItemSpec, u64)>) -> &mut Self {
        for menu_item in menu_items.iter() {
            self.menu_items.push(menu_item.clone());
        }
        self
    }

    pub fn id_field(&mut self, id_field: i32) -> &mut Self {
        self.id_field = id_field;
        self
    }

    pub fn selection_mode(&mut self, selection_mode: gtk::SelectionMode) -> &mut Self {
        self.selection_mode = selection_mode;
        self
    }

    pub fn build<R: RawDataSource + 'static>(&self, raw_data_source: R) -> BufferedListView<R> {
        let list_store = BufferedListStore::new(raw_data_source);
        let view = gtk::TreeViewBuilder::new().headers_visible(true).build();
        view.set_model(Some(list_store.list_store()));
        view.get_selection().set_mode(self.selection_mode);

        for col in list_store.columns() {
            view.append_column(&col);
        }

        let popup_menu = ManagedMenuBuilder::new()
            .selection(&view.get_selection())
            .build();

        let blv = BufferedListView(Rc::new(BufferedListViewCore {
            view,
            list_store,
            selected_id: RefCell::new(None),
            popup_menu,
            callbacks: RefCell::new(HashMap::new()),
            id_field: self.id_field,
        }));

        for (name, menu_item_spec, condns) in self.menu_items.iter() {
            let blv_c = blv.clone();
            let name_c = (*name).to_string();
            blv.0
                .popup_menu
                .append_item(name, menu_item_spec, *condns)
                .connect_activate(move |_| blv_c.menu_item_selected(&name_c));
            blv.0
                .callbacks
                .borrow_mut()
                .insert((*name).to_string(), vec![]);
        }

        let blv_c = blv.clone();
        blv.0.view.connect_button_press_event(move |_, event| {
            if event.get_event_type() == gdk::EventType::ButtonPress {
                match event.get_button() {
                    2 => {
                        blv_c.0.view.get_selection().unselect_all();
                        gtk::Inhibit(true)
                    }
                    3 => {
                        blv_c.set_selected_id(event.get_position());
                        blv_c.0.popup_menu.popup_at_event(event);
                        return gtk::Inhibit(true);
                    }
                    _ => gtk::Inhibit(false),
                }
            } else {
                gtk::Inhibit(false)
            }
        });

        blv
    }
}
