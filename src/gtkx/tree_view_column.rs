// Copyright 2017 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use gtk;
use gtk::prelude::*;

pub fn simple_text_column(
    title: &str,
    text_id: i32,
    sort_id: i32,
    bg_id: i32,
    fg_id: i32,
    fixed_width: i32,
    resizable: bool,
) -> gtk::TreeViewColumn {
    let col = gtk::TreeViewColumn::new();
    col.set_title(title);
    col.set_resizable(resizable);
    if sort_id >= 0 {
        col.set_sort_column_id(sort_id)
    };
    col.set_fixed_width(fixed_width);
    let cell = gtk::CellRendererText::new();
    cell.set_property_editable(false);
    col.pack_start(&cell, resizable);
    if text_id >= 0 {
        col.add_attribute(&cell, "text", text_id)
    };
    if bg_id >= 0 {
        col.add_attribute(&cell, "background-rgba", bg_id)
    };
    if fg_id >= 0 {
        col.add_attribute(&cell, "foreground-rgba", fg_id)
    };
    col
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
