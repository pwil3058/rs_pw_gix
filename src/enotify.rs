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

use std::cell::{Cell, RefCell};
use std::default::Default;
use std::sync::Mutex;

pub struct EventNotifier {
    callbacks: Mutex<RefCell<Vec<(u64, u64, Box<Fn(u64)>)>>>,
    next_token: Mutex<Cell<u64>>,
}

impl Default for EventNotifier {
    fn default() -> Self {
        EventNotifier {
            callbacks: Mutex::new(RefCell::new(Vec::new())),
            next_token: Mutex::new(Cell::new(0)),
        }
    }
}

impl EventNotifier {
    pub fn add_notification_cb(&self, events: u64, callback: Box<Fn(u64)>) -> u64 {
        let next_token = self.next_token.lock().unwrap();
        let token = next_token.get();
        next_token.set(token + 1);

        let callbacks = self.callbacks.lock().unwrap();
        callbacks
            .borrow_mut()
            .push((token, events, callback));

        token
    }

    pub fn del_notification(&self, token: u64) {
        let callbacks = self.callbacks.lock().unwrap();
        let position = callbacks.borrow().iter().position(|x| x.0 == token);
        if let Some(position) = position {
            callbacks.borrow_mut().remove(position);
        }
    }

    pub fn notify_events(&self, events: u64) {
        let callbacks = self.callbacks.lock().unwrap();
        for (_, registered_events, callback) in callbacks.borrow().iter() {
            if registered_events & events != 0 {
                callback(events)
            }
        }
    }
}

//lazy_static! {
    //static ref ENOTIFIER: EventNotifier = { EventNotifier::default() };
//}

//pub fn add_notification_cb(events: u64, callback: fn(u64)) -> u64 {
    //ENOTIFIER.add_notification_cb(events, callback)
//}

//pub fn del_notification(token: u64) {
    //ENOTIFIER.del_notification(token)
//}

//pub fn notify_events(token: u64) {
    //ENOTIFIER.notify_events(token)
//}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
