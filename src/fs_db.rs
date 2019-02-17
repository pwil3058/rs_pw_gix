// Copyright 2019 Peter Williams <pwil3058@gmail.com>
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

//! File system database to feed file tree stores/views

pub use crate::gtkx::value::Row;

pub trait FsObjectIfce {
    fn tree_store_spec() -> Vec<gtk::Type>;
    fn row_is_a_dir(row: &Row) -> bool;
    fn get_name_from_row(row: &Row) -> &str;
    fn get_path_from_row(row: &Row) -> &str;

    fn row_is_the_same(&self, row: &Row) -> bool;
    fn name(&self) -> &str;
    fn path(&self) -> &str;
    fn is_dir(&self) -> bool;
    fn row(&self) -> &Row;
}

pub trait FsDbIfce<DOI, FOI>
where
    DOI: FsObjectIfce,
    FOI: FsObjectIfce,
{
    fn new() -> Self;

    fn dir_contents(
        &self,
        dir_path: &str,
        show_hidden: bool,
        hide_clean: bool,
    ) -> (Vec<DOI>, Vec<FOI>);

    fn is_current(&self) -> bool {
        true
    }

    fn reset(&mut self);
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
