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
                .child(
                    TextBlock::new()
                        .text("Hello, World")
                        .v_align("center")
                        .h_align("center")
                        .build(ctx),
                )
                .build(ctx)
        })
        .run()
}
