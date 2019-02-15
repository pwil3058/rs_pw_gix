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

#[macro_export]
macro_rules! are_equal_as {
    ( $v1:expr, $v2:expr, $t:ty ) => {{
        assert_eq!($v1.type_(), $v2.type_());
        // TODO: panic if extracted values are None
        let v1: Option<$t> = $v1.get();
        let v2: Option<$t> = $v2.get();
        v1 == v2
    }};
}

#[macro_export]
macro_rules! are_eq_values {
    ( $v1:expr, $v2:expr ) => {{
        match $v1.type_() {
            gtk::Type::I8 => are_equal_as!($v1, $v2, i8),
            gtk::Type::U8 => are_equal_as!($v1, $v2, u8),
            gtk::Type::Bool => are_equal_as!($v1, $v2, bool),
            gtk::Type::I32 => are_equal_as!($v1, $v2, i32),
            gtk::Type::U32 => are_equal_as!($v1, $v2, u32),
            gtk::Type::I64 => are_equal_as!($v1, $v2, i64),
            gtk::Type::U64 => are_equal_as!($v1, $v2, u64),
            gtk::Type::F32 => are_equal_as!($v1, $v2, f32),
            gtk::Type::F64 => are_equal_as!($v1, $v2, f64),
            gtk::Type::String => are_equal_as!($v1, $v2, String),
            _ => panic!("operation not defined for: {:?}", $v1.type_()),
        }
    }};
}

pub type Row = Vec<gtk::Value>;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
