// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//! Provide mechanisms to control the sensitivity and/or visibility
//! of groups of widgets dependent on a widget and/or an application's
//! current state.
//! Up to 128 conditions can be used to describe a state.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ops::{Add, BitAnd, BitOr, Not},
};

use gtk::prelude::*;

/// Set of boolean flags representing conditions that define a state
#[derive(Debug, Default, PartialOrd, PartialEq, Ord, Eq, Clone, Copy)]
pub struct Condns(pub u128);

pub const DONT_CARE: Condns = Condns(0);

impl Condns {
    /// Returns `true` if `self`'s flags are a subset of `other`' flags.
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.0 & other.0 == self.0
    }

    /// Returns `true` if `self`'s flags are a superset of `other`'s flags.
    pub fn is_superset_of(&self, other: &Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl From<u128> for Condns {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl Not for Condns {
    type Output = Self;

    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl BitAnd for Condns {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl BitOr for Condns {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl Add<&Change> for Condns {
    type Output = Self;

    fn add(self, change: &Change) -> Self {
        change.1 | (self & !change.0)
    }
}

/// An ordered pair of `Condns` specifying a change to be made to a set of conditions.
/// The first member of the pair specifies which conditions should be changed and the second
/// specifies the values that they should be given.
#[derive(Debug, Default, PartialOrd, PartialEq, Ord, Eq, Clone, Copy)]
pub struct Change(pub Condns, pub Condns);

impl Change {
    #[inline]
    pub fn changed_condns(&self) -> Condns {
        self.0
    }

    #[inline]
    pub fn new_values(&self) -> Condns {
        self.1
    }

    pub fn is_valid(&self) -> bool {
        self.changed_condns().is_superset_of(&self.new_values())
    }
}

pub trait ApplyChange {
    fn apply_changed_condns(&self, change: &Change);
}

/// Conditions for a `Widget` to be sensitive, visible and/or both.
#[derive(Debug, Clone, Copy)]
pub enum Policy {
    Sensitivity(Condns),
    Visibility(Condns),
    Both(Condns, Condns),
}

#[derive(Debug, Default)]
pub struct Enforcer {
    widget_policy: RefCell<HashMap<gtk::Widget, Policy>>,
    current_condns: Cell<Condns>,
}

impl Enforcer {
    pub fn with_initial_condns(init_condns: Condns) -> Self {
        let enforcer = Enforcer::default();
        enforcer.current_condns.set(init_condns);
        enforcer
    }

    pub fn add_widget<W: IsA<gtk::Widget>>(&self, w: &W, policy: Policy) {
        let widget = w.clone().upcast::<gtk::Widget>();
        match &policy {
            Policy::Sensitivity(condns) => {
                widget.set_sensitive(self.current_condns.get().is_superset_of(condns))
            }
            Policy::Visibility(condns) => {
                widget.set_visible(self.current_condns.get().is_superset_of(condns))
            }
            Policy::Both(s_condns, v_condns) => {
                widget.set_sensitive(self.current_condns.get().is_superset_of(s_condns));
                widget.set_visible(self.current_condns.get().is_superset_of(v_condns))
            }
        }
        let result = self.widget_policy.borrow_mut().insert(widget, policy);
        debug_assert!(result.is_none());
    }
}

impl ApplyChange for Enforcer {
    fn apply_changed_condns(&self, change: &Change) {
        debug_assert!(change.is_valid());
        let new_condns = self.current_condns.get() + change;
        for (widget, policy) in self.widget_policy.borrow().iter() {
            match policy {
                Policy::Sensitivity(condns) => {
                    widget.set_sensitive(new_condns.is_superset_of(condns))
                }
                Policy::Visibility(condns) => widget.set_visible(new_condns.is_superset_of(condns)),
                Policy::Both(s_condns, v_condns) => {
                    widget.set_sensitive(new_condns.is_superset_of(s_condns));
                    widget.set_visible(new_condns.is_superset_of(v_condns))
                }
            }
        }
        self.current_condns.set(new_condns);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn into() {
        assert_eq!(Condns(45), 45.into());
    }

    #[test]
    fn subset_of() {
        assert!(Condns(0b101010).is_subset_of(&Condns(0b101010)));
        assert!(Condns(0b101010).is_subset_of(&Condns(0b101110)));
        assert!(!Condns(0b1101010).is_subset_of(&Condns(0b101110)));
    }

    #[test]
    fn superset_of() {
        assert!(!Condns(0b101010).is_superset_of(&Condns(0b101110)));
        assert!(!Condns(0b1101010).is_superset_of(&Condns(0b101110)));
        assert!(Condns(0b1101010).is_superset_of(&Condns(0b101010)));
        assert!(Condns(0b1101010).is_superset_of(&Condns(0b1101010)));
    }
}
