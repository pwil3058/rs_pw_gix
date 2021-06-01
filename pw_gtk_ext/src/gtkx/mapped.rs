// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RequiredMapAction {
    Renew,
    Update,
    Nothing,
}

pub trait MapManagedUpdate {
    fn do_renew(&self);
    fn do_update(&self);
    fn is_mapped(&self) -> bool;
    fn get_required_map_action(&self) -> RequiredMapAction;
    fn set_required_map_action(&self, action: RequiredMapAction);

    fn auto_update(&self) {
        match self.get_required_map_action() {
            RequiredMapAction::Nothing => self.update(),
            _ => (),
        }
    }

    fn on_map_action(&self) {
        match self.get_required_map_action() {
            RequiredMapAction::Renew => {
                self.renew();
                self.set_required_map_action(RequiredMapAction::Nothing);
            }
            RequiredMapAction::Update => {
                self.update();
                self.set_required_map_action(RequiredMapAction::Nothing);
            }
            RequiredMapAction::Nothing => (),
        }
    }

    fn renew(&self) {
        if self.is_mapped() {
            self.do_renew();
            self.set_required_map_action(RequiredMapAction::Nothing)
        } else {
            self.set_required_map_action(RequiredMapAction::Renew)
        }
    }

    fn update(&self) {
        if self.is_mapped() {
            self.do_update();
            self.set_required_map_action(RequiredMapAction::Nothing)
        } else if self.get_required_map_action() != RequiredMapAction::Renew {
            self.set_required_map_action(RequiredMapAction::Update)
        }
    }
}
