// Copyright 2017 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use pw_gtk_ext::gtk::prelude::*;
use pw_gtk_ext::wrapper::*;
use pw_gtk_ext::*;

#[derive(PWO2)]
struct SimpleCore {
    h_box: gtk::Box,
}

#[derive(PWO2)]
struct Simple(std::rc::Rc<SimpleCore>);

fn main() {
    if gtk::init().is_err() {
        println!("Gtk++ failed to initialize!");
        return;
    };

    let simple_core = SimpleCore {
        h_box: gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Horizontal)
            .build(),
    };
    let v_box = gtk::BoxBuilder::new()
        .orientation(gtk::Orientation::Vertical)
        .build();
    v_box.pack_start(simple_core.pwo(), false, false, 0);
    println!("PWO: {:?}", simple_core.pwo());
}
