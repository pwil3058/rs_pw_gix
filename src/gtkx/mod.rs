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

#[macro_use]
pub mod value;
#[macro_use]
pub mod list_store;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
