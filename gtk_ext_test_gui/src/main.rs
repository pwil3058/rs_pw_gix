// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::rc;

use pw_gtk_ext::gtk::prelude::*;
use pw_gtk_ext::gtkx::check_button::MutuallyExclusiveCheckButtonsBuilder;
use pw_gtk_ext::gtkx::combo_box_text::SortedUnique;
use pw_gtk_ext::gtkx::list_store::{ListRowOps, ListViewSpec, WrappedListStore, WrappedTreeModel};
use pw_gtk_ext::gtkx::menu::ManagedMenuBuilder;
use pw_gtk_ext::gtkx::notebook::TabRemoveLabelBuilder;
use pw_gtk_ext::gtkx::radio_button::RadioButtonsBuilder;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum XYZ {
    X,
    Y,
    Z,
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
    mecbs.connect_changed(move |tag| {
        let selected = mecbs_c.selected();
        assert_eq!(tag, selected);
    });
    v_box.pack_start(mecbs.pwo(), false, false, 0);
    v_box.pack_start(
        &gtk::Separator::new(gtk::Orientation::Horizontal),
        false,
        false,
        0,
    );

    let mecbs = MutuallyExclusiveCheckButtonsBuilder::new()
        .orientation(gtk::Orientation::Horizontal)
        .check_button(XYZ::X, "--x", "just testing: x")
        .check_button(XYZ::Y, "--y", "just testing: y")
        .check_button(XYZ::Z, "--z", "just testing: z")
        .build();
    let mecbs_c = mecbs.clone();
    mecbs.connect_changed(move |tag| {
        let selected = mecbs_c.selected();
        assert_eq!(tag, selected);
    });
    v_box.pack_start(mecbs.pwo(), false, false, 0);

    let radio_buttons = RadioButtonsBuilder::new()
        .radio_button(XYZ::X, "--X", "X radio button")
        .radio_button(XYZ::Y, "--Y", "Y oprtio")
        .radio_button(XYZ::Z, "--Z", "Z optio")
        .default(XYZ::Y)
        .build();
    let radio_buttons_c = radio_buttons.clone();
    radio_buttons.connect_changed(move |tag| {
        let selected = radio_buttons_c.selected();
        assert_eq!(tag, selected);
    });
    v_box.pack_start(radio_buttons.pwo(), false, false, 0);

    let cbt = gtk::ComboBoxText::new();
    assert!(cbt.remove_text_item("one").is_err());
    assert_eq!(cbt.insert_text_item("one").unwrap(), -1);
    assert_eq!(cbt.insert_text_item("two").unwrap(), -1);
    assert_eq!(cbt.insert_text_item("three").unwrap(), 1);
    assert_eq!(cbt.insert_text_item("four").unwrap(), 0);
    assert_eq!(cbt.insert_text_item("five").unwrap(), 0);
    assert_eq!(cbt.insert_text_item("six").unwrap(), 3);
    assert_eq!(cbt.insert_text_item("zero").unwrap(), -1);
    assert!(cbt.remove_text_item("two").is_ok());
    assert!(cbt.remove_text_item("two").is_err());
    assert!(cbt.remove_text_item("four").is_ok());
    assert!(cbt.remove_text_item("four").is_err());
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
    v_box.pack_start(&cbt, false, false, 0);

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

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    v_box.pack_start(&hbox, false, false, 0);
    let menu_bar = gtk::MenuBarBuilder::new().build();
    menu_bar.show();
    hbox.pack_start(&menu_bar, true, true, 0);
    let menu_item = gtk::MenuItemBuilder::new().label("Menu").build();
    let menu1 = ManagedMenuBuilder::new()
        .items(&[
            ("remove", ("Remove", None, Some("help help")).into(), 0),
            ("delete", ("Delete", None, None).into(), 0),
        ])
        .build();
    menu_item.set_submenu(Some(menu1.pwo()));
    menu_bar.add(&menu_item);
    menu1
        .append_item("add", &("Add", None, Some("help message")).into(), 0)
        .unwrap()
        .connect_activate(|_| println!("add"));
    menu1
        .menu_item("remove")
        .expect("unknown item")
        .connect_activate(|_| println!("remove"));
    menu1
        .menu_item("delete")
        .expect("unknown item")
        .connect_activate(|_| println!("delete"));
    menu_bar.show_all();

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
