// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

#[macro_use]
pub mod value;
#[macro_use]
pub mod tree_model;
#[macro_use]
pub mod list_store;
#[macro_use]
pub mod tree_store;
#[macro_use]
pub mod tree_view;

pub mod coloured;
pub mod combo_box_text;
pub mod dialog;
pub mod drawing_area;
pub mod entry;
pub mod menu;
pub mod notebook;
pub mod paned;
pub mod tree_view_column;
pub mod window;

pub mod icon_size {
    pub const MENU: i32 = 16;
    pub const SMALL_TOOLBAR: i32 = 16;
    pub const LARGE_TOOLBAR: i32 = 24;
    pub const BUTTON: i32 = 16;
    pub const DND: i32 = 32;
    pub const DIALOG: i32 = 48;
}
