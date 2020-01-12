// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::rc::Rc;

use gtk;
use gtk::{BoxExt, ContainerExt, WidgetExt};

use pw_gix::gtkx::check_button::MutuallyExclusiveCheckButtonsBuilder;
use pw_gix::recollections;
use pw_gix::wrapper::*;

fn main() {
    recollections::init("./.recollections");
    if gtk::init().is_err() {
        println!("Gtk++ failed to initialize!");
        return;
    };
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
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

    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
