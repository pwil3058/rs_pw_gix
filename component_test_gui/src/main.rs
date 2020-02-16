// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{cell::RefCell, rc::Rc};

use glib;
use gtk;
use gtk::prelude::*;
use gtk::{BoxExt, ContainerExt, WidgetExt};

use pw_gix::{
    colour::*,
    gdk_pixbufx::viewer::*,
    glibx::*,
    gtkx::{
        check_button::MutuallyExclusiveCheckButtonsBuilder,
        combo_box_text::SortedUnique,
        entry::{RGBEntryInterface, RGBHexEntryBox},
        list_store::*,
        window::RememberGeometry,
    },
    recollections,
    wrapper::*,
};

fn main() {
    recollections::init("./.recollections");
    if gtk::init().is_err() {
        println!("Gtk++ failed to initialize!");
        return;
    };
    let win = gtk::Window::new(gtk::WindowType::Toplevel);

    test_list_store_simple_row_ops();
    test_list_store_row_buffer();

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

    let rgb_entry_box = RGBHexEntryBox::create();
    let rgb = rgb_entry_box.get_rgb();
    println!("{:?} {:?}", rgb, RGB::BLACK);
    assert_eq!(rgb, RGB::BLACK);
    vbox.pack_start(&rgb_entry_box.pwo(), false, false, 0);

    let button = gtk::Button::new_with_label("Image Viewer");
    vbox.pack_start(&button, false, false, 0);
    button.connect_clicked(|_| launch_image_viewer());

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
