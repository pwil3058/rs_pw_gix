// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::gtk::WidgetExt;

pub use pw_gtk_ext_derive::*;

use crate::gtkx::dialog_user::DialogUser;

pub trait PackableWidgetObject {
    type PWT: glib::IsA<gtk::Widget> + WidgetExt;

    fn pwo(&self) -> Self::PWT;
}

// TODO: figure out how to do this with a derive procedural macro
#[macro_export]
macro_rules! implement_rc_wrappped_pwo {
    ( $widget:ty, $pwo:ty ) => {
        impl PackableWidgetObject for $widget {
            type PWT = $pwo;

            fn pwo(&self) -> Self::PWT {
                self.0.pwo()
            }
        }
    };
}

pub trait WidgetWrapper: PackableWidgetObject + DialogUser {}

#[cfg(test)]
mod wrapper_tests {
    use super::*;
    use crate::gtkx::dialog_user::*;
    use glib::Cast;
    use gtk;
    use std::rc::Rc;

    #[test]
    fn widget_wrapper_simple() {
        #[derive(PWO, Wrapper)]
        struct _TestWrapper {
            vbox: gtk::Box,
        }
    }

    #[test]
    fn widget_rc_wrapper_simple() {
        #[derive(PWO)]
        struct _TestWrapperCore {
            vbox: gtk::Dialog,
        }

        #[derive(Wrapper)]
        struct _TestWrapper(Rc<_TestWrapperCore>);

        implement_rc_wrappped_pwo!(_TestWrapper, gtk::Dialog);
    }

    #[test]
    fn widget_wrapper_generic_simple() {
        #[derive(PWO, Wrapper)]
        struct _TestWrapperCore<A, B, C> {
            vbox: gtk::Box,
            _a: A,
            _b: B,
            _c: C,
        }
    }

    #[test]
    fn widget_wrapper_generic_constrained() {
        #[derive(PWO, Wrapper)]
        struct _TestWrapper<A, B, C>
        where
            A: Eq,
            C: PartialEq,
        {
            vbox: gtk::Box,
            _a: A,
            _b: B,
            _c: C,
        }
    }

    #[test]
    fn widget_wrapper_complex_generic_constrained() {
        trait Alpha<B>
        where
            B: PartialEq,
        {
        }
        #[derive(PWO, Wrapper)]
        struct _TestWrapper<A, B>
        where
            A: Alpha<B>,
            B: PartialEq,
        {
            vbox: gtk::Box,
            _a: A,
            _b: B,
        }
    }
}
