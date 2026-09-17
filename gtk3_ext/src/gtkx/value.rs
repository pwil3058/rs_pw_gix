// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

#[macro_export]
macro_rules! are_equal_as {
    ( $v1:expr, $v2:expr, $t:ty ) => {{
        debug_assert_eq!($v1.type_(), $v2.type_());
        // TODO: panic if extracted values are None
        let v1 = $v1.get::<$t>().expect(UNEXPECTED);
        let v2 = $v2.get::<$t>().expect(UNEXPECTED);
        v1 == v2
    }};
}

#[macro_export]
macro_rules! are_eq_values {
    ( $v1:expr, $v2:expr ) => {{
        match $v1.type_() {
            glib::Type::I8 => are_equal_as!($v1, $v2, i8),
            glib::Type::U8 => are_equal_as!($v1, $v2, u8),
            glib::Type::BOOL => are_equal_as!($v1, $v2, bool),
            glib::Type::I32 => are_equal_as!($v1, $v2, i32),
            glib::Type::U32 => are_equal_as!($v1, $v2, u32),
            glib::Type::I64 => are_equal_as!($v1, $v2, i64),
            glib::Type::U64 => are_equal_as!($v1, $v2, u64),
            glib::Type::F32 => are_equal_as!($v1, $v2, f32),
            glib::Type::F64 => are_equal_as!($v1, $v2, f64),
            glib::Type::STRING => are_equal_as!($v1, $v2, String),
            _ => panic!("operation not defined for: {:?}", $v1.type_()),
        }
    }};
}
