// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    gdk,
    glib,
    // glibx::prelude::GlibValueExt,
    gtk::prelude::{
        GtkListStoreExt, GtkMenuItemExt, TreeModelExt, TreeSelectionExt, TreeViewExt, WidgetExt,
    },
    gtkx::{
        list_store::*,
        menu::{ManagedMenu, MenuItemSpec},
    },
    sav_state::*,
    wrapper::*,
};

type PopupCallback = Box<dyn Fn(Option<String>, Option<Vec<String>>)>;

#[derive(PWO)]
pub struct ListViewWithPopUpMenu {
    view: gtk::TreeView,
    list_store: gtk::ListStore,
    selected_id: RefCell<Option<String>>,
    popup_menu: ManagedMenu,
    callbacks: RefCell<HashMap<String, Vec<PopupCallback>>>,
    id_field: i32,
}

pub trait ListViewSpec {
    fn column_types(&self) -> Vec<glib::Type>;
    fn columns(&self) -> Vec<gtk::TreeViewColumn>;
}

impl ListViewWithPopUpMenu {
    fn set_selected_id(&self, posn: (f64, f64)) {
        if let Some(location) = self.view.path_at_pos(posn.0 as i32, posn.1 as i32)
            && let Some(path) = location.0
            && let Some(list_store) = self.view.model()
            && let Some(iter) = list_store.iter(&path)
        {
            let value = list_store.value(&iter, self.id_field);
            if let Some(string) = value.get().unwrap() {
                *self.selected_id.borrow_mut() = Some(string);
                self.popup_menu.update_hover_condns(true);
                return;
            }
        };
        *self.selected_id.borrow_mut() = None;
        self.popup_menu.update_hover_condns(false);
    }

    pub fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        self.popup_menu.update_condns(changed_condns)
    }

    pub fn connect_popup_menu_item<F: Fn(Option<String>, Option<Vec<String>>) + 'static>(
        &self,
        name: &str,
        callback: F,
    ) {
        self.callbacks
            .borrow_mut()
            .get_mut(name)
            .expect("invalid name")
            .push(Box::new(callback));
    }

    fn menu_item_selected(&self, name: &str) {
        let hovered_id = (*self.selected_id.borrow())
            .as_ref()
            .map(|id| id.to_string());
        let selection = self.view.selection();
        let (tree_paths, store) = selection.selected_rows();
        let selected_ids: Option<Vec<String>> = if !tree_paths.is_empty() {
            let mut vector = vec![];
            for tree_path in tree_paths.iter() {
                if let Some(iter) = store.iter(tree_path)
                    && let Ok(id) = store.value(&iter, self.id_field).get::<String>()
                {
                    vector.push(id);
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

    pub fn add_row(&self, row: &[glib::Value]) {
        self.list_store.append_row(row);
    }

    pub fn remove_row(&self, id: &str) {
        if let Some((_, iter)) = self.list_store.find_row_where(|list_store, iter| {
            list_store.value(iter, self.id_field).get() == Ok(Some(id))
        }) {
            self.list_store.remove(&iter);
        } else {
            panic!("{}: id not found", id);
        }
    }

    pub fn remove_all(&self) {
        self.list_store.clear();
    }
}

pub struct ListViewWithPopUpMenuBuilder {
    menu_items: Vec<(&'static str, MenuItemSpec, u64)>,
    id_field: i32,
    selection_mode: gtk::SelectionMode,
}

impl Default for ListViewWithPopUpMenuBuilder {
    fn default() -> Self {
        Self {
            menu_items: vec![],
            id_field: 0,
            selection_mode: gtk::SelectionMode::None,
        }
    }
}

impl ListViewWithPopUpMenuBuilder {
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

    pub fn build(&self, spec: &impl ListViewSpec) -> Rc<ListViewWithPopUpMenu> {
        let list_store = gtk::ListStore::new(&spec.column_types());
        let view = gtk::TreeView::builder().headers_visible(true).build();
        view.set_model(Some(&list_store));
        view.selection().set_mode(self.selection_mode);

        for col in spec.columns() {
            view.append_column(&col);
        }

        let popup_menu = ManagedMenu::builder().selection(&view.selection()).build();

        let rgb_l_v = Rc::new(ListViewWithPopUpMenu {
            view,
            list_store,
            selected_id: RefCell::new(None),
            popup_menu,
            callbacks: RefCell::new(HashMap::new()),
            id_field: self.id_field,
        });

        for (name, menu_item_spec, condns) in self.menu_items.iter() {
            let rgb_l_v_c = Rc::clone(&rgb_l_v);
            let name_c = (*name).to_string();
            match rgb_l_v
                .popup_menu
                .append_item(name, menu_item_spec, *condns)
            {
                Ok(menu_item) => {
                    menu_item.connect_activate(move |_| rgb_l_v_c.menu_item_selected(&name_c))
                }
                Err(err) => panic!("Error building menu item '{}':: {}", name, err),
            };
            // rgb_l_v
            //     .popup_menu
            //     .append_item(name, menu_item_spec, *condns)
            //     .connect_activate(move |_| rgb_l_v_c.menu_item_selected(&name_c));
            rgb_l_v
                .callbacks
                .borrow_mut()
                .insert((*name).to_string(), vec![]);
        }

        let rgb_l_v_c = Rc::clone(&rgb_l_v);
        rgb_l_v.view.connect_button_press_event(move |_, event| {
            if event.event_type() == gdk::EventType::ButtonPress {
                match event.button() {
                    2 => {
                        rgb_l_v_c.view.selection().unselect_all();
                        glib::Propagation::Stop
                        // gkk::gtk::Inhibit(true)
                    }
                    3 => {
                        rgb_l_v_c.set_selected_id(event.position());
                        rgb_l_v_c.popup_menu.popup_at_event(event);
                        glib::Propagation::Stop
                        // gtk::Inhibit(true)
                    }
                    _ => glib::Propagation::Proceed, // _ => gtk::Inhibit(false),
                }
            } else {
                glib::Propagation::Proceed
                // gtk::Inhibit(false)
            }
        });

        rgb_l_v
    }
}
