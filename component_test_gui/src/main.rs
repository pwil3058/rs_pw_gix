// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cell::RefCell, rc::Rc};

use pw_gix::{
    //cairo,
    gdk_pixbufx::viewer::*,
    glib,
    glibx::*,
    gtk::{self, prelude::*, BoxExt, ContainerExt, WidgetExt},
    gtkx::{
        check_button::MutuallyExclusiveCheckButtonsBuilder,
        combo_box_text::SortedUnique,
        entry::HexEntryBuilder,
        list::{ListViewSpec, ListViewWithPopUpMenuBuilder},
        list_store::*,
        menu_ng::ManagedMenuBuilder,
        window::RememberGeometry,
    },
    recollections,
    wrapper::*,
};

mod sav_test;

use pw_gix::sav_state::{SAV_SELN_UNIQUE, SAV_SELN_UNIQUE_OR_HOVER_OK};
use sav_test::SavTest;

struct TestListSpec;

impl ListViewSpec for TestListSpec {
    fn column_types(&self) -> Vec<glib::Type> {
        vec![glib::Type::String, glib::Type::String]
    }

    fn columns(&self) -> Vec<gtk::TreeViewColumn> {
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

    test_list_store_simple_row_ops();
    test_list_store_row_buffer();
    //let surface = cairo::ImageSurface::create_from_png(&mut reader).unwrap();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let mecbs = MutuallyExclusiveCheckButtonsBuilder::new()
        .orientation(gtk::Orientation::Vertical)
        .check_button("a", "--a", "just testing: a")
        .check_button("b", "--b", "just testing: b")
        .check_button("c", "--c", "just testing: c")
        .build();
    let mecbs_c = Rc::clone(&mecbs);
    mecbs.connect_changed(move |name| {
        let selected = mecbs_c.selected();
        assert_eq!(name, selected);
    });
    vbox.pack_start(&mecbs.pwo(), false, false, 0);

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
    vbox.pack_start(&cbt, false, false, 0);

    let hex_entry = HexEntryBuilder::<u8>::new().editable(true).build();
    vbox.pack_start(&hex_entry.pwo(), false, false, 0);

    let button = gtk::Button::with_label("Image Viewer");
    vbox.pack_start(&button, false, false, 0);
    button.connect_clicked(|_| launch_image_viewer());

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    vbox.pack_start(&hbox, false, false, 0);
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
    menu_item.set_submenu(Some(&menu1.pwo()));
    menu_bar.add(&menu_item);
    menu1
        .append_item("add", &("Add", None, Some("help message")).into(), 0)
        .connect_activate(|_| println!("add"));
    menu1
        .menu_item("remove")
        .expect("unknown item")
        .connect_activate(|_| println!("remove"));
    menu_bar.show_all();

    let sav_test = Rc::new(SavTest::new());
    vbox.pack_start(&sav_test.pwo(), false, false, 0);

    let button = gtk::Button::with_label("Cancel/Ok Question");
    vbox.pack_start(&button, false, false, 0);
    let sav_test_c = Rc::clone(&sav_test);
    button.connect_clicked(move |_| {
        let response = sav_test_c.ask_confirm_action("Do you really want to?", None);
        let msg = format!("Response: {:?}", response);
        sav_test_c.inform_user(&msg, None);
    });

    let list = ListViewWithPopUpMenuBuilder::new()
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
        .build(&TestListSpec);
    vbox.pack_start(&list.pwo(), true, true, 0);
    list.connect_popup_menu_item("edit", |s, l| println!("edit: {:?} : {:?}", s, l));
    list.connect_popup_menu_item("remove", |s, l| println!("remove: {:?} : {:?}", s, l));
    list.add_row(&vec!["one".to_value(), "two".to_value()]);
    list.add_row(&vec!["three".to_value(), "four".to_value()]);

    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}

macro_rules! are_equal_as {
    ( $v1:expr, $v2:expr, $t:ty ) => {{
        assert_eq!($v1.type_(), $v2.type_());
        // TODO: panic if extracted values are None
        let v1: Option<$t> = $v1.get_ok();
        let v2: Option<$t> = $v2.get_ok();
        v1 == v2
    }};
}

macro_rules! are_eq_values {
    ( $v1:expr, $v2:expr ) => {{
        match $v1.type_() {
            glib::Type::I8 => are_equal_as!($v1, $v2, i8),
            glib::Type::U8 => are_equal_as!($v1, $v2, u8),
            glib::Type::Bool => are_equal_as!($v1, $v2, bool),
            glib::Type::I32 => are_equal_as!($v1, $v2, i32),
            glib::Type::U32 => are_equal_as!($v1, $v2, u32),
            glib::Type::I64 => are_equal_as!($v1, $v2, i64),
            glib::Type::U64 => are_equal_as!($v1, $v2, u64),
            glib::Type::F32 => are_equal_as!($v1, $v2, f32),
            glib::Type::F64 => are_equal_as!($v1, $v2, f64),
            glib::Type::String => are_equal_as!($v1, $v2, String),
            _ => panic!("operation not defined for: {:?}", $v1.type_()),
        }
    }};
}

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

fn test_list_store_simple_row_ops() {
    let test_list_store =
        gtk::ListStore::new(&[glib::Type::String, glib::Type::String, glib::Type::String]);
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

fn test_list_store_row_buffer() {
    struct TestBuffer {
        id: u8,
        row_buffer_core: Rc<RefCell<RowBufferCore<Vec<String>>>>,
    }

    impl TestBuffer {
        pub fn new() -> TestBuffer {
            let row_buffer_core = RowBufferCore::<Vec<String>>::default();
            let buf = TestBuffer {
                id: 0,
                row_buffer_core: Rc::new(RefCell::new(row_buffer_core)),
            };
            buf.init();
            buf
        }

        pub fn set_id(&mut self, value: u8) {
            self.id = value;
        }
    }

    impl RowBuffer<Vec<String>> for TestBuffer {
        fn get_core(&self) -> Rc<RefCell<RowBufferCore<Vec<String>>>> {
            self.row_buffer_core.clone()
        }

        fn set_raw_data(&self) {
            let mut core = self.row_buffer_core.borrow_mut();
            match self.id {
                0 => {
                    core.set_raw_data(Vec::new(), Vec::new());
                }
                1 => {
                    core.set_raw_data(
                        vec!["one".to_string(), "two".to_string(), "three".to_string()],
                        vec![1, 2, 3],
                    );
                }
                _ => {
                    core.set_raw_data(Vec::new(), Vec::new());
                }
            }
        }

        fn finalise(&self) {
            let mut core = self.row_buffer_core.borrow_mut();
            let mut rows: Vec<Row> = Vec::new();
            for item in core.raw_data.iter() {
                rows.push(vec![item.to_value()]);
            }
            core.rows = Rc::new(rows);
            core.set_is_finalised_true();
        }
    }

    let mut buffer = TestBuffer::new();

    assert_eq!(buffer.get_rows().len(), 0);
    assert!(buffer.needs_init());
    assert!(!buffer.needs_finalise());
    assert!(buffer.is_current());

    buffer.set_id(1);
    assert!(!buffer.is_current());
    assert_eq!(buffer.get_rows().len(), 0);
    buffer.finalise();
    assert!(buffer.is_current());
    let rows = buffer.get_rows();
    assert_eq!(rows[0][0].get_ok(), Some("one"));
    assert_eq!(rows[1][0].get_ok(), Some("two"));
    assert_eq!(rows[2][0].get_ok(), Some("three"));
    assert_eq!(rows[0][0].get_ok_some::<String>(), "one".to_string());
    assert_eq!(rows[1][0].get_ok_some::<&str>(), "two");
    assert_eq!(rows[2][0].get_ok_some::<&str>(), "three");
}

fn launch_image_viewer() {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_geometry_from_recollections("image_viewer", (200, 200));
    window.set_destroy_with_parent(true);
    window.set_title("component_test_gui: Image Viewer");

    let view = PixbufViewBuilder::new().load_last_image(true).build();
    window.add(&view.pwo());
    window.show_all();

    window.present();
}
