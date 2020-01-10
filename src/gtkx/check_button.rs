// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::collections::HashMap;

use gtk::prelude::*;

use crate::wrapper::*;

pub struct MutuallyExclusiveCheckButtons {
    box_: gtk::Box,
    check_buttons: HashMap<String, gtk::CheckButton>,
}

impl_widget_wrapper!(box_: gtk::Box, MutuallyExclusiveCheckButtons);

pub struct MutuallyExclusiveCheckButtonsBuilder {
    check_buttons: Vec<(String, String)>,
    orientation: gtk::Orientation,
}

impl MutuallyExclusiveCheckButtonsBuilder {
    pub fn new() -> MutuallyExclusiveCheckButtonsBuilder {
        MutuallyExclusiveCheckButtonsBuilder {
            check_buttons: vec![],
            orientation: gtk::Orientation::Horizontal,
        }
    }
}
