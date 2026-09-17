// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use crate::glib::value::{FromValue, FromValueOptional, Value};

pub trait GlibValueExt {
    fn get_ok<'a, T: FromValue<'a>>(&'a self) -> Option<T>;
    fn get_ok_some<'a, T: FromValueOptional<'a>>(&'a self) -> T;
    fn get_some_ok<'a, T: FromValue<'a>>(&'a self) -> T;
}

impl GlibValueExt for Value {
    fn get_ok<'a, T: FromValue<'a>>(&'a self) -> Option<T> {
        self.get::<T>().expect("Programmer Error: type mismatch")
    }

    fn get_ok_some<'a, T: FromValueOptional<'a>>(&'a self) -> T {
        self.get::<T>()
            .expect("Programmer Error: type mismatch")
            .expect("Programmer Error: unexpecte 'None'")
    }

    fn get_some_ok<'a, T: FromValue<'a>>(&'a self) -> T {
        self.get_some::<T>()
            .expect("Programmer Error: type mismatch")
    }
}
