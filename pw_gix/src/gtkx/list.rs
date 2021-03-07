// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    gtk::{
        GtkListStoreExt, GtkMenuItemExt, TreeModelExt, TreeSelectionExt, TreeViewExt, WidgetExt,
    },
    gtkx::{
        list_store::*,
        menu_ng::{ManagedMenu, ManagedMenuBuilder, MenuItemSpec},
    },
    sav_state::*,
    wrapper::*,
};

type PopupCallback = Box<dyn Fn(&str)>;

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
        if let Some(location) = self.view.get_path_at_pos(posn.0 as i32, posn.1 as i32) {
            if let Some(path) = location.0 {
                if let Some(list_store) = self.view.get_model() {
                    if let Some(iter) = list_store.get_iter(&path) {
                        let value = list_store.get_value(&iter, self.id_field);
                        if let Some(string) = value.get().unwrap() {
                            *self.selected_id.borrow_mut() = Some(string);
                            self.popup_menu.update_hover_condns(true);
                            return;
                        }
                    }
                }
            }
        };
        *self.selected_id.borrow_mut() = None;
        self.popup_menu.update_hover_condns(false);
    }

    pub fn update_popup_condns(&self, changed_condns: MaskedCondns) {
        self.popup_menu.update_condns(changed_condns)
    }

    pub fn connect_popup_menu_item<F: Fn(&str) + 'static>(&self, name: &str, callback: F) {
        self.callbacks
            .borrow_mut()
            .get_mut(name)
            .expect("invalid name")
            .push(Box::new(callback));
    }

    fn menu_item_selected(&self, name: &str) {
        if let Some(ref id) = *self.selected_id.borrow() {
            for callback in self
                .callbacks
                .borrow()
                .get(name)
                .expect("invalid name")
                .iter()
            {
                callback(&id)
            }
        }
    }

    pub fn add_row(&self, row: &[glib::Value]) {
        self.list_store.append_row(&row.to_vec());
    }

    pub fn remove_row(&self, id: &str) {
        if let Some((_, iter)) = self
            .list_store
            .find_row_where(|list_store, iter| list_store.get_value(iter, 0).get_ok() == Some(id))
        {
            self.list_store.remove(&iter);
        } else {
            panic!("{}: id not found", id);
        }
    }

    pub fn remove_all(&self) {
        self.list_store.clear();
    }
}

#[derive(Default)]
pub struct ListViewWithPopUpMenuBuilder {
    menu_items: Vec<(&'static str, MenuItemSpec, u64)>,
    id_field: i32,
}

impl ListViewWithPopUpMenuBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn menu_item(&mut self, menu_item: (&'static str, MenuItemSpec, u64)) -> &mut Self {
        self.menu_items.push(menu_item);
        self
    }

    pub fn id_field(&mut self, id_field: i32) -> &mut Self {
        self.id_field = id_field;
        self
    }

    pub fn build(&self, spec: &impl ListViewSpec) -> Rc<ListViewWithPopUpMenu> {
        let list_store = gtk::ListStore::new(&spec.column_types());
        let view = gtk::TreeViewBuilder::new().headers_visible(true).build();
        view.set_model(Some(&list_store));
        view.get_selection().set_mode(gtk::SelectionMode::None);

        for col in spec.columns() {
            view.append_column(&col);
        }

        let rgb_l_v = Rc::new(ListViewWithPopUpMenu {
            view,
            list_store,
            selected_id: RefCell::new(None),
            popup_menu: ManagedMenuBuilder::new().build(),
            callbacks: RefCell::new(HashMap::new()),
            id_field: self.id_field,
        });

        for (name, menu_item_spec, condns) in self.menu_items.iter() {
            let rgb_l_v_c = Rc::clone(&rgb_l_v);
            let name_c = (*name).to_string();
            rgb_l_v
                .popup_menu
                .append_item(name, menu_item_spec, *condns)
                .connect_activate(move |_| rgb_l_v_c.menu_item_selected(&name_c));
            rgb_l_v
                .callbacks
                .borrow_mut()
                .insert((*name).to_string(), vec![]);
        }

        let rgb_l_v_c = Rc::clone(&rgb_l_v);
        rgb_l_v.view.connect_button_press_event(move |_, event| {
            if event.get_event_type() == gdk::EventType::ButtonPress && event.get_button() == 3 {
                rgb_l_v_c.set_selected_id(event.get_position());
                rgb_l_v_c.popup_menu.popup_at_event(event);
                return gtk::Inhibit(true);
            };
            gtk::Inhibit(false)
        });

        rgb_l_v
    }
}
