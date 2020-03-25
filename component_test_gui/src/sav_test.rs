// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::rc::Rc;

use pw_gix::{
    sav::{ApplyChange, Change, Condns, Enforcer, Policy},
    wrapper::*,
};

use gtk::prelude::*;

#[derive(PWO)]
pub struct SavTest {
    vbox: gtk::Box,
    enforcer: Rc<Enforcer>,
}

const SAV_A_ACTIVE: Condns = Condns(8);
const SAV_A_INACTIVE: Condns = Condns(8 << 1);
const SAV_B_ACTIVE: Condns = Condns(8 << 2);
const SAV_B_INACTIVE: Condns = Condns(8 << 3);

impl Default for SavTest {
    fn default() -> Self {
        Self {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            enforcer: Rc::new(Enforcer::default()),
        }
    }
}

impl SavTest {
    pub fn with_initial_condns(init_condns: Condns) -> Self {
        Self {
            vbox: gtk::Box::new(gtk::Orientation::Vertical, 0),
            enforcer: Rc::new(Enforcer::with_initial_condns(init_condns)),
        }
    }
}

impl SavTest {
    pub fn new() -> Self {
        let sav_test = Self::with_initial_condns(SAV_A_INACTIVE | SAV_B_INACTIVE);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        sav_test.vbox.pack_start(&hbox, false, false, 0);

        let button_a = gtk::ButtonBuilder::new().label("A").build();
        hbox.pack_start(&button_a, false, false, 0);
        sav_test
            .enforcer
            .add_widget(&button_a, Policy::Sensitivity(SAV_A_ACTIVE));
        assert!(sav_test.enforcer.remove_widget(&button_a).is_ok());
        sav_test
            .enforcer
            .add_widget(&button_a, Policy::Sensitivity(SAV_A_ACTIVE));

        let check_button_b = gtk::CheckButtonBuilder::new().label("B not A").build();
        hbox.pack_start(&check_button_b, false, false, 0);
        sav_test.enforcer.add_widget(
            &check_button_b,
            Policy::Sensitivity(SAV_A_INACTIVE | SAV_B_ACTIVE),
        );

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        sav_test.vbox.pack_start(&hbox, false, false, 0);

        let check_button_a = gtk::CheckButtonBuilder::new().label("A").build();
        hbox.pack_start(&check_button_a, false, false, 0);
        let enforcer_c = Rc::clone(&sav_test.enforcer);
        check_button_a.connect_toggled(move |cb| {
            if cb.get_active() {
                enforcer_c
                    .apply_changed_condns(&Change(SAV_A_ACTIVE | SAV_A_INACTIVE, SAV_A_ACTIVE));
            } else {
                enforcer_c
                    .apply_changed_condns(&Change(SAV_A_ACTIVE | SAV_A_INACTIVE, SAV_A_INACTIVE));
            }
        });

        let check_button_b = gtk::CheckButtonBuilder::new().label("B").build();
        hbox.pack_start(&check_button_b, false, false, 0);
        let enforcer_c = Rc::clone(&sav_test.enforcer);
        check_button_b.connect_toggled(move |cb| {
            if cb.get_active() {
                enforcer_c
                    .apply_changed_condns(&Change(SAV_B_ACTIVE | SAV_B_INACTIVE, SAV_B_ACTIVE));
            } else {
                enforcer_c
                    .apply_changed_condns(&Change(SAV_B_ACTIVE | SAV_B_INACTIVE, SAV_B_INACTIVE));
            }
        });

        sav_test
    }
}
