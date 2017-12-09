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

use std::cell::{Cell, RefCell};
use std::convert::From;
use std::rc::Rc;

use gdk;
use gtk;
use gtk::prelude::*;

use geometry::*;

pub struct XYSelectionCore {
    drawing_area: gtk::DrawingArea,
    start_xy: Cell<Option<Point>>,
    end_xy: Cell<Option<Point>>,
    selection_made: Cell<bool>,
    selection_made_callbacks: RefCell<Vec<Box<Fn()>>>,
}

impl XYSelectionCore {
    pub fn in_progress(&self) -> bool {
        !(self.start_xy.get().is_none() || self.selection_made.get())
    }

    pub fn selection_made(&self) -> bool {
        self.selection_made.get()
    }

    pub fn is_drawable(&self) -> bool {
        self.start_xy.get().is_some()
    }

    pub fn reset(&self) {
        self.selection_made.set(false);
        self.start_xy.set(None);
        self.end_xy.set(None);
        //self.emit("status-changed", False)
    }

    pub fn get_selected_rectangle(&self, scale: f64) -> Option<Rectangle<f64>> {
        if let Some(raw_start) = self.start_xy.get() {
            if let Some(raw_end) = self.end_xy.get() {
                let start = raw_start * scale;
                let end = raw_end * scale;
                let delta = end - start;
                // width and height have to be positive
                let (x, width) = if delta.x() >= 0.0 {
                    (start.x(), delta.x())
                } else {
                    (end.x(), -delta.x())
                };
                let (y, height) = if delta.y() >= 0.0 {
                    (start.y(), delta.y())
                } else {
                    (end.y(), -delta.y())
                };
                Some(Rectangle{x, y, width, height})
            } else {
                panic!("File: {:?} Line: {:?}: should NOT happen", file!(), line!());
            }
        } else {
            None
        }
    }

    pub fn connect_selection_made<F: 'static + Fn()>(&self, callback: F) {
        self.selection_made_callbacks.borrow_mut().push(Box::new(callback))
    }
}

pub type XYSelection = Rc<XYSelectionCore>;

pub trait XYSelectionInterface {
    fn create(drawing_area: &gtk::DrawingArea) -> XYSelection;
}

impl XYSelectionInterface for XYSelection {
    fn create(drawing_area: &gtk::DrawingArea) -> XYSelection {
        let events = gdk::EventMask::POINTER_MOTION_MASK | gdk::EventMask::BUTTON_PRESS_MASK |
            gdk::EventMask::BUTTON_RELEASE_MASK | gdk::EventMask::LEAVE_NOTIFY_MASK;
        drawing_area.add_events(events.bits() as i32);
        let xys = Rc::new(
            XYSelectionCore {
                drawing_area: drawing_area.clone(),
                start_xy: Cell::new(None),
                end_xy: Cell::new(None),
                selection_made: Cell::new(false),
                selection_made_callbacks: RefCell::new(Vec::new()),
            }
        );
        let xys_c = xys.clone();
        xys.drawing_area.connect_button_press_event(
            move |da, event| {
                if event.get_button() == 1 && !event.get_state().contains(gdk::ModifierType::CONTROL_MASK) {
                    let point = Point::from(event.get_position());
                    xys_c.start_xy.set(Some(point));
                    xys_c.end_xy.set(Some(point));
                    xys_c.selection_made.set(false);
                    da.queue_draw();
                    gtk::Inhibit(true)
                } else if event.get_button() == 2 {
                    if xys_c.in_progress() {
                        xys_c.reset()
                    } else if xys_c.selection_made.get() {
                        xys_c.reset()
                    };
                    da.queue_draw();
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            }
        );
        let xys_c = xys.clone();
        xys.drawing_area.connect_button_release_event(
            move |da, event| {
                if event.get_button() == 1 && xys_c.in_progress() {
                    xys_c.end_xy.set(Some(Point::from(event.get_position())));
                    xys_c.selection_made.set(true);
                    for callback in xys_c.selection_made_callbacks.borrow().iter() {
                        callback();
                    }
                    da.queue_draw();
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            }
        );
        let xys_c = xys.clone();
        xys.drawing_area.connect_motion_notify_event(
            move |da,event| {
                if xys_c.in_progress() {
                    xys_c.end_xy.set(Some(Point::from(event.get_position())));
                    da.queue_draw();
                    gtk::Inhibit(true)
                } else {
                    gtk::Inhibit(false)
                }
            }
        );
        let xys_c = xys.clone();
        xys.drawing_area.connect_leave_notify_event(
            move |da, _| {
                if xys_c.in_progress() {
                    xys_c.reset();
                    da.queue_draw();
                };
                gtk::Inhibit(false)
            }
        );

        xys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
