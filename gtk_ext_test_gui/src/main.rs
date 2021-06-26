// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::rc;

use pw_gtk_ext::gtk::prelude::*;
use pw_gtk_ext::gtkx::check_button::MutuallyExclusiveCheckButtonsBuilder;
use pw_gtk_ext::gtkx::list_store::{ListRowOps, ListViewSpec, WrappedListStore, WrappedTreeModel};
use pw_gtk_ext::gtkx::notebook::TabRemoveLabelBuilder;
use pw_gtk_ext::gtkx::tree_view::TreeViewWithPopupBuilder;
use pw_gtk_ext::pw_recollect::recollections;
use pw_gtk_ext::sav_state::{SAV_SELN_UNIQUE, SAV_SELN_UNIQUE_OR_HOVER_OK};
use pw_gtk_ext::wrapper::*;
use pw_gtk_ext::*;

#[derive(PWO)]
struct SimpleCore {
    h_box: gtk::Box,
}

#[derive(PWO, Wrapper, WClone)]
struct Simple(std::rc::Rc<SimpleCore>);

struct TestListSpec;

impl ListViewSpec for TestListSpec {
    fn column_types() -> Vec<glib::Type> {
        vec![glib::Type::String, glib::Type::String]
    }

    fn columns() -> Vec<gtk::TreeViewColumn> {
        let mut cols = vec![];

        let col = gtk::TreeViewColumnBuilder::new()
            .title("Id")
            .resizable(false)
            .sort_column_id(0)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "text", 0);
        cols.push(col);

        let col = gtk::TreeViewColumnBuilder::new()
            .title("Name")
            .resizable(true)
            .sort_column_id(1)
            .sort_indicator(true)
            .build();
        let cell = gtk::CellRendererTextBuilder::new().editable(false).build();
        col.pack_start(&cell, false);
        col.add_attribute(&cell, "text", 1);
        cols.push(col);

        cols
    }
}

fn main() {
    recollections::init("./.recollections");
    if gtk::init().is_err() {
        println!("Gtk++ failed to initialize!");
        return;
    };
    let win = gtk::Window::new(gtk::WindowType::Toplevel);

    let simple_core = SimpleCore {
        h_box: gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Horizontal)
            .build(),
    };

    let v_box = gtk::BoxBuilder::new()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let mecbs = MutuallyExclusiveCheckButtonsBuilder::new()
        .orientation(gtk::Orientation::Vertical)
        .check_button("a", "--a", "just testing: a")
        .check_button("b", "--b", "just testing: b")
        .check_button("c", "--c", "just testing: c")
        .build();
    let mecbs_c = mecbs.clone();
    mecbs.connect_changed(move |name| {
        let selected = mecbs_c.selected();
        assert_eq!(name, selected);
    });
    v_box.pack_start(mecbs.pwo(), false, false, 0);

    v_box.pack_start(simple_core.pwo(), false, false, 0);
    let simple = Simple(rc::Rc::new(SimpleCore {
        h_box: gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Horizontal)
            .build(),
    }));
    let notebook = gtk::NotebookBuilder::new().build();
    let tab_label = TabRemoveLabelBuilder::new().label_text("whatever").build();
    let menu_label = gtk::Label::new(Some("whatever"));
    notebook.insert_page_menu(
        simple.pwo(),
        Some(tab_label.pwo()),
        Some(&menu_label),
        Some(1),
    );
    v_box.pack_start(&notebook, false, false, 0);

    let list_store = WrappedListStore::<TestListSpec>::new();

    let list = TreeViewWithPopupBuilder::new()
        .selection_mode(gtk::SelectionMode::Multiple)
        .menu_item((
            "edit",
            ("Edit", None, Some("Edit the indicated paint.")).into(),
            SAV_SELN_UNIQUE_OR_HOVER_OK,
        ))
        .menu_item((
            "remove",
            ("Remove", None, Some("Remove the indicated row.")).into(),
            SAV_SELN_UNIQUE,
        ))
        .id_field(1)
        .build(&list_store);
    v_box.pack_start(list.pwo(), true, true, 0);
    list.connect_popup_menu_item("edit", |s, l| println!("edit: {:?} : {:?}", s, l));
    list.connect_popup_menu_item("remove", |s, l| println!("remove: {:?} : {:?}", s, l));
    list_store
        .model()
        .append_row(&vec!["one".to_value(), "two".to_value()]);
    list_store
        .model()
        .append_row(&vec!["three".to_value(), "four".to_value()]);

    v_box.show_all();
    win.add(&v_box);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
