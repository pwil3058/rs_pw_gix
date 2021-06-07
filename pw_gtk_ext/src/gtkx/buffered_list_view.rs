// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::glib::Value;
use crate::gtk::prelude::IsA;
use crate::gtkx::buffered_list_store::{BufferedListStore, RowDataSource};
use crate::gtkx::menu::{ManagedMenu, ManagedMenuBuilder, MenuItemSpec};
use crate::sav_state::MaskedCondns;
use crate::sourceview::prelude::{
    GtkMenuItemExt, TreeModelExt, TreeSelectionExt, TreeViewExt, WidgetExt,
};
use crate::wrapper::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type PopupCallback = Box<dyn Fn(Option<Value>, Vec<Value>)>;
type DoubleClickCallback = Box<dyn Fn(&Value)>;

#[derive(PWO)]
pub struct BufferedListViewCore<R: RowDataSource> {
    view: gtk::TreeView,
    list_store: BufferedListStore<R>,
    selected_id: RefCell<Option<Value>>,
    popup_menu: ManagedMenu,
    popup_callbacks: RefCell<HashMap<String, Vec<PopupCallback>>>,
    double_click_callbacks: RefCell<Vec<DoubleClickCallback>>,
    id_field: i32,
}

#[derive(PWO, WClone)]
pub struct BufferedListView<R: RowDataSource>(Rc<BufferedListViewCore<R>>);

impl<R: RowDataSource> BufferedListView<R> {
    fn get_id_value_at(&self, posn: (f64, f64)) -> Option<Value> {
        if let Some(location) = self.0.view.get_path_at_pos(posn.0 as i32, posn.1 as i32) {
            if let Some(path) = location.0 {
                if let Some(list_store) = self.0.view.get_model() {
                    if let Some(iter) = list_store.get_iter(&path) {
                        let value = list_store.get_value(&iter, self.0.id_field);
                        return Some(value);
                    }
                }
            }
        };
        None
    }

    fn set_selected_id(&self, posn: (f64, f64)) {
        match self.get_id_value_at(posn) {
            Some(value) => {
                *self.0.selected_id.borrow_mut() = Some(value);
                self.0.popup_menu.update_hover_condns(true);
            }
            None => {
                *self.0.selected_id.borrow_mut() = None;
                self.0.popup_menu.update_hover_condns(false);
            }
        }
    }

    pub fn connect_popup_menu_item<F: Fn(Option<Value>, Vec<Value>) + 'static>(
        &self,
        name: &str,
        callback: F,
    ) {
        self.0
            .popup_callbacks
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
        let mut selected_ids = vec![];
        for tree_path in tree_paths.iter() {
            if let Some(iter) = store.get_iter(&tree_path) {
                selected_ids.push(store.get_value(&iter, self.0.id_field));
            }
        }
        if hovered_id.is_some() || !selected_ids.is_empty() {
            for callback in self
                .0
                .popup_callbacks
                .borrow()
                .get(name)
                .expect("invalid name")
                .iter()
            {
                callback(hovered_id.clone(), selected_ids.clone())
            }
        }
    }

    pub fn connect_double_click<F: Fn(&Value) + 'static>(&self, callback: F) {
        self.0
            .double_click_callbacks
            .borrow_mut()
            .push(Box::new(callback));
    }

    fn process_double_click(&self, posn: (f64, f64)) {
        if let Some(value) = self.get_id_value_at(posn) {
            for callback in self.0.double_click_callbacks.borrow().iter() {
                callback(&value)
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
    tree_view_builder: gtk::TreeViewBuilder,
}

impl Default for BufferedListViewBuilder {
    fn default() -> Self {
        Self {
            menu_items: vec![],
            id_field: 0,
            selection_mode: gtk::SelectionMode::Single,
            tree_view_builder: gtk::TreeViewBuilder::new(),
        }
    }
}

macro_rules! impl_builder_option {
    ( $name:ident, $type:ty ) => {
        pub fn $name(mut self, $name: $type) -> Self {
            self.tree_view_builder = self.tree_view_builder.$name($name);
            self
        }
    };
}

impl BufferedListViewBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn menu_item(mut self, menu_item: (&'static str, MenuItemSpec, u64)) -> Self {
        self.menu_items.push(menu_item);
        self
    }

    pub fn menu_items(mut self, menu_items: &[(&'static str, MenuItemSpec, u64)]) -> Self {
        for menu_item in menu_items.iter() {
            self.menu_items.push(menu_item.clone());
        }
        self
    }

    pub fn id_field(mut self, id_field: i32) -> Self {
        self.id_field = id_field;
        self
    }

    pub fn selection_mode(mut self, selection_mode: gtk::SelectionMode) -> Self {
        self.selection_mode = selection_mode;
        self
    }

    /// Wrappers for TreeViewBuilder options relevant to lists.
    pub fn hadjustment<P: IsA<gtk::Adjustment>>(mut self, hadjustment: &P) -> Self {
        self.tree_view_builder = self.tree_view_builder.hadjustment(hadjustment);
        self
    }

    pub fn vadjustment<P: IsA<gtk::Adjustment>>(mut self, vadjustment: &P) -> Self {
        self.tree_view_builder = self.tree_view_builder.vadjustment(vadjustment);
        self
    }

    impl_builder_option!(activate_on_single_click, bool);
    impl_builder_option!(border_width, u32);
    impl_builder_option!(enable_grid_lines, gtk::TreeViewGridLines);
    impl_builder_option!(enable_search, bool);
    impl_builder_option!(events, gdk::EventMask);
    impl_builder_option!(fixed_height_mode, bool);
    impl_builder_option!(halign, gtk::Align);
    impl_builder_option!(headers_clickable, bool);
    impl_builder_option!(headers_visible, bool);
    impl_builder_option!(height_request, i32);
    impl_builder_option!(hover_expand, bool);
    impl_builder_option!(hover_selection, bool);
    impl_builder_option!(hscroll_policy, gtk::ScrollablePolicy);
    impl_builder_option!(margin, i32);
    impl_builder_option!(margin_bottom, i32);
    impl_builder_option!(margin_end, i32);
    impl_builder_option!(margin_start, i32);
    impl_builder_option!(margin_top, i32);
    impl_builder_option!(name, &str);
    impl_builder_option!(opacity, f64);
    impl_builder_option!(rubber_banding, bool);
    impl_builder_option!(search_column, i32);
    impl_builder_option!(sensitive, bool);
    impl_builder_option!(tooltip_column, i32);
    impl_builder_option!(tooltip_markup, &str);
    impl_builder_option!(tooltip_text, &str);
    impl_builder_option!(valign, gtk::Align);
    impl_builder_option!(visible, bool);
    impl_builder_option!(vscroll_policy, gtk::ScrollablePolicy);
    impl_builder_option!(width_request, i32);

    pub fn build<R: RowDataSource + 'static>(self, raw_data_source: R) -> BufferedListView<R> {
        let list_store = BufferedListStore::new(raw_data_source);
        let view = self.tree_view_builder.build();
        view.set_model(Some(list_store.list_store()));
        view.get_selection().set_mode(self.selection_mode);

        for col in R::columns() {
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
            popup_callbacks: RefCell::new(HashMap::new()),
            double_click_callbacks: RefCell::new(vec![]),
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
                .popup_callbacks
                .borrow_mut()
                .insert((*name).to_string(), vec![]);
        }

        let blv_c = blv.clone();
        blv.0
            .view
            .connect_button_press_event(move |_, event| match event.get_event_type() {
                gdk::EventType::ButtonPress => match event.get_button() {
                    2 => {
                        blv_c.0.view.get_selection().unselect_all();
                        gtk::Inhibit(true)
                    }
                    3 => {
                        blv_c.set_selected_id(event.get_position());
                        blv_c.0.popup_menu.popup_at_event(event);
                        gtk::Inhibit(true)
                    }
                    _ => gtk::Inhibit(false),
                },
                gdk::EventType::DoubleButtonPress => match event.get_button() {
                    1 => {
                        blv_c.process_double_click(event.get_position());
                        gtk::Inhibit(true)
                    }
                    _ => gtk::Inhibit(false),
                },
                _ => gtk::Inhibit(false),
            });

        blv
    }
}
