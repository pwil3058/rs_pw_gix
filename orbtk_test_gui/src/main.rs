// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use orbtk::prelude::*;

fn main() {
    orbtk::initialize();

    Application::new()
        .window(|ctx| {
            Window::new()
                .title("Hello, OrbTk!")
                .position((2000.0, 1000.0))
                .size(200.0, 100.0)
                .resizeable(true)
                .child(
                    TextBlock::new()
                        .text("Hello, World")
                        .v_align("center")
                        .h_align("center")
                        .build(ctx),
                )
                .child(
                    ComboBox::new()
                        .count(5)
                        .items_builder(move |bc, index| {
                            let text = format!("item{}", index);
                            TextBlock::new().v_align("center").text(text).build(bc)
                        })
                        .selected_index(0)
                        .on_changed("selected_index", |_states, entity| {
                            println!("Entity: {:?}", entity);
                        })
                        .build(ctx),
                )
                .build(ctx)
        })
        .run()
}
