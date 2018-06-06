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

use std::clone;
use std::convert;
use std::error::{self, Error};
use std::fmt;
use std::io;
use std::process::{Command};

use gdk::{self, WindowExtManual};
use gdk_pixbuf::Pixbuf;
use gtk::{self, ClipboardExt};

use which::which;

#[derive(Debug)]
pub enum FailureReason {
    UserCancelled,
    NoDefaultScreen,
    NonCompositing,
    NoRGBAVisual,
    NoDeviceManager,
    PointerNotFound,
    PointerGrabFailed(gdk::GrabStatus),
    KeyboardNotFound,
    KeyboardGrabFailed(gdk::GrabStatus),
    IOError(io::Error),
}

impl clone::Clone for FailureReason {
    // NB: this is necessary because io::Error doesn't implement copy OR clone
    fn clone(&self) -> FailureReason {
        match *self {
            FailureReason::UserCancelled => FailureReason::UserCancelled,
            FailureReason::NoDefaultScreen => FailureReason::NoDefaultScreen,
            FailureReason::NonCompositing => FailureReason::NonCompositing,
            FailureReason::NoRGBAVisual => FailureReason::NoRGBAVisual,
            FailureReason::NoDeviceManager => FailureReason::NoDeviceManager,
            FailureReason::PointerNotFound => FailureReason::PointerNotFound,
            FailureReason::PointerGrabFailed(status) => FailureReason::PointerGrabFailed(status),
            FailureReason::KeyboardNotFound => FailureReason::KeyboardNotFound,
            FailureReason::KeyboardGrabFailed(status) => FailureReason::KeyboardGrabFailed(status),
            FailureReason::IOError(ref error) => {
                let kind = error.kind();
                let description = error.description().clone();
                let cloned_error = io::Error::new(kind, description);
                FailureReason::IOError(cloned_error)
            },
        }
    }
}

#[derive(Debug)]
pub struct Failure {
    reason: FailureReason,
    message: String,
}

impl Failure {
    pub fn new(reason: FailureReason) -> Failure {
        let message = match reason {
            FailureReason::UserCancelled => "User cancelled".to_string(),
            FailureReason::NoDefaultScreen => "No default screen".to_string(),
            FailureReason::NonCompositing => "Non compositing screen".to_string(),
            FailureReason::NoRGBAVisual => "No RGBA visual".to_string(),
            FailureReason::NoDeviceManager => "No device manager".to_string(),
            FailureReason::PointerNotFound => "Pointer not found".to_string(),
            FailureReason::PointerGrabFailed(_) => "Pointer grab failed".to_string(),
            FailureReason::KeyboardNotFound => "Keyboard not found".to_string(),
            FailureReason::KeyboardGrabFailed(_) => "Keyboard grab failed".to_string(),
            FailureReason::IOError(ref error) => {
                let description = error.description().clone();
                format!("I/O Error: {}", description)
            },
        };
        Failure{reason, message}
    }

    pub fn reason(&self) -> FailureReason {
        self.reason.clone()
    }

    pub fn user_cancelled(&self) -> bool {
        match self.reason {
            FailureReason::UserCancelled => true,
            _ => false,
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Failure {
    fn description(&self) -> &str {
        &self.message
    }
}

impl convert::From<io::Error> for Failure {
    fn from(io_error: io::Error) -> Failure {
        Failure::new(FailureReason::IOError(io_error))
    }
}

pub fn screen_sampling_available() -> bool {
    area_selection::is_available() || which("gnome-screenshot").is_ok()
}

pub fn take_screen_sample() -> Result<(), Failure> {
    let result = area_selection::select_area();
    match result {
        Ok(ref rectangle) => {
            if let Some(pixbuf) = get_screen_pixbuf_rectangle(rectangle) {
                let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                cbd.set_image(&pixbuf);
            } else {
                Command::new("gnome-screenshot").arg("-ac").spawn()?;
            }
        },
        Err(err) => if !err.user_cancelled() {
            Command::new("gnome-screenshot").arg("-ac").spawn()?;
        }
    };
    Ok(())
}

pub fn get_screen_pixbuf_rectangle(rect: &gdk::Rectangle) -> Option<Pixbuf> {
    let root_window = gdk::Window::get_default_root_window();
    root_window.get_pixbuf(rect.x, rect.y, rect.width, rect.height)
}

pub mod area_selection {
    use super::{Failure, FailureReason};

    use std::cell::{Cell};
    use std::convert::From;
    use std::rc::Rc;

    use cairo;
    use gdk::{self, DeviceExt, DeviceManagerExt, DisplayExt, ScreenExt, WindowExt, WindowExtManual};
    use gio;
    use gtk::{self, GtkWindowExt, WidgetExt};

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct IntPoint {
        x: i32,
        y: i32,
    }

    impl From<(f64, f64)> for IntPoint {
        fn from(xy: (f64, f64)) -> IntPoint {
            IntPoint{x: xy.0 as i32, y: xy.1 as i32}
        }
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct IntSize {
        width: i32,
        height: i32,
    }

    impl From<(IntPoint, IntPoint)> for IntSize {
        fn from(ipp: (IntPoint, IntPoint)) -> IntSize {
            IntSize{
                width: (ipp.0.x - ipp.1.x).abs(),
                height: (ipp.0.y - ipp.1.y).abs(),
            }
        }
    }

    fn top_left_corner(ipp: (IntPoint, IntPoint)) -> IntPoint {
        IntPoint{
            x: ipp.0.x.min(ipp.1.x),
            y: ipp.0.y.min(ipp.1.y),
        }
    }

    #[derive(Debug)]
    struct SelectAreaDataCore {
        start_position: Cell<Option<(f64, f64)>>,
        current_position: Cell<Option<(f64, f64)>>,
        end_position: Cell<Option<(f64, f64)>>,
        button_num: Cell<Option<u32>>,
        window: gtk::Window,
    }

    impl SelectAreaDataCore {
        fn new() -> SelectAreaDataCore {
            SelectAreaDataCore{
                start_position: Cell::new(None),
                current_position: Cell::new(None),
                end_position: Cell::new(None),
                button_num: Cell::new(None),
                window: gtk::Window::new(gtk::WindowType::Popup),
            }
        }

        fn is_makeable() -> bool {
            if let Some(screen) = gdk::Screen::get_default() {
                if screen.is_composited() {
                    return screen.get_rgba_visual().is_some();
                }
            };
            false
        }

        fn in_progress(&self) -> bool {
            self.button_num.get().is_some()
        }

        fn in_progress_for(&self, button_num: u32) -> bool {
            if let Some(my_button_num) = self.button_num.get() {
                my_button_num == button_num
            } else {
                false
            }
        }

        fn finish(&self) -> Result<gdk::Rectangle, Failure> {
            self.window.destroy();
            if let Some(start_position) = self.start_position.get() {
                if let Some(end_position) = self.end_position.get() {
                    let i_start : IntPoint = start_position.into();
                    let i_last : IntPoint = end_position.into();
                    let position = top_left_corner((i_start, i_last));
                    let size = IntSize::from((i_start, i_last));
                    return Ok(gdk::Rectangle{
                        x: position.x,
                        y: position.y,
                        width: size.width,
                        height: size.height,
                    });
                }
            };
            Err(Failure::new(FailureReason::UserCancelled))
        }
    }

    type SelectAreaData = Rc<SelectAreaDataCore>;

    trait SelectAreaDataIfce {
        fn create() -> Result<SelectAreaData, Failure>;
    }

    impl SelectAreaDataIfce for SelectAreaData {
        fn create() -> Result<SelectAreaData, Failure> {
            let sad = Rc::new(SelectAreaDataCore::new());

            if let Some(screen) = gdk::Screen::get_default() {
                if screen.is_composited() {
                    if let Some(ref visual) = screen.get_rgba_visual() {
                        sad.window.set_visual(visual);
                        sad.window.set_app_paintable(true);
                    } else {
                        return Err(Failure::new(FailureReason::NoRGBAVisual))
                    }
                } else {
                    return Err(Failure::new(FailureReason::NonCompositing))
                }
            } else {
                return Err(Failure::new(FailureReason::NoDefaultScreen))
            }
            let events = gdk::EventMask::KEY_PRESS_MASK | gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK | gdk::EventMask::BUTTON_MOTION_MASK;
            sad.window.add_events(events.bits() as i32);

            let sad_c = sad.clone();
            sad.window.connect_draw(
                move |_, cairo_context| {
                    if let Some(start_position) = sad_c.start_position.get() {
                        if let Some(current_position) = sad_c.current_position.get() {
                            // NB. draw OUSIDE the selected area so that we don't have
                            // an issue with how long it takes the screen to be redrawn
                            // after we finish and before a sample is taken.
                            let lw = 2.0;
                            cairo_context.set_line_width(lw);
                            let x = start_position.0.min(current_position.0) - lw;
                            let y = start_position.1.min(current_position.1) - lw;
                            let width = (start_position.0 - current_position.0).abs() + 2.0 * lw;
                            let height = (start_position.1 - current_position.1).abs() + 2.0 * lw;
                            cairo_context.rectangle(x, y, width, height);
                            cairo_context.set_source_rgb(0.0, 0.0, 0.0);
                            cairo_context.set_dash(&[3.0], 0.0);
                            cairo_context.set_operator(cairo::enums::Operator::Xor);
                            cairo_context.stroke();
                        }
                    };
                    gio::signal::Inhibit(false)
                }
            );

            let sad_c = sad.clone();
            sad.window.connect_key_press_event(
                move |_, event| {
                    if event.get_keyval() == gdk::enums::key::Escape {
                        sad_c.start_position.set(None);
                        sad_c.current_position.set(None);
                        sad_c.end_position.set(None);
                        sad_c.button_num.set(None);

                        gtk::main_quit();
                    }

                    gio::signal::Inhibit(true)
                }
            );

            let sad_c = sad.clone();
            sad.window.connect_button_press_event(
                move |_, event| {
                    if !sad_c.in_progress() {
                        sad_c.button_num.set(Some(event.get_button()));
                        sad_c.start_position.set(Some(event.get_position()));
                    }

                    gio::signal::Inhibit(true)
                }
            );

            let sad_c = sad.clone();
            sad.window.connect_button_release_event(
                move |window, event| {
                    if sad_c.in_progress_for(event.get_button()) {
                        sad_c.end_position.set(Some(event.get_position()));
                        sad_c.current_position.set(None);
                        sad_c.button_num.set(None);
                        window.queue_draw();

                        gtk::main_quit();
                    }

                    gio::signal::Inhibit(true)
                }
            );

            let sad_c = sad.clone();
            sad.window.connect_motion_notify_event(
                move |window, event| {
                    if sad_c.in_progress() {
                        sad_c.current_position.set(Some(event.get_position()));
                        window.queue_draw();
                    }

                    gio::signal::Inhibit(true)
                }
            );

            let root_window = gdk::Window::get_default_root_window();

            sad.window.move_(0, 0);
            sad.window.resize(root_window.get_width(), root_window.get_height());
            sad.window.show();

            Ok(sad)
        }
    }

    #[derive(Debug)]
    struct PointerAndKeyboard {
        pointer: gdk::Device,
        keyboard: gdk::Device,
    }

    impl PointerAndKeyboard {
        fn is_makeable() -> bool {
            if let Some(display) = gdk::Display::get_default() {
                if let Some(manager) = display.get_device_manager() {
                    if let Some(pointer) = manager.get_client_pointer() {
                        return pointer.get_associated_device().is_some();
                    }
                }
            };
            false
        }

        fn grab<W: WidgetExt>(w: &W) -> Result<PointerAndKeyboard, Failure> {
            if let Some(display) = gdk::Display::get_default() {
                if let Some(manager) = display.get_device_manager() {
                    if let Some(pointer) = manager.get_client_pointer() {
                        if let Some(keyboard) = pointer.get_associated_device() {
                            if let Some(ref window) = w.get_window() {
                                let cursor = gdk::Cursor::new_for_display(&display, gdk::CursorType::Crosshair);
                                let status = pointer.grab(
                                    window,
                                    gdk::GrabOwnership::None,
                                    false,
                                    gdk::EventMask::POINTER_MOTION_MASK|
                                    gdk::EventMask::BUTTON_PRESS_MASK|
                                    gdk::EventMask::BUTTON_RELEASE_MASK,
                                    Some(&cursor),
                                    0,
                                );
                                if status != gdk::GrabStatus::Success {
                                    return Err(Failure::new(FailureReason::PointerGrabFailed(status)));
                                }
                                let status = keyboard.grab(
                                    window,
                                    gdk::GrabOwnership::None,
                                    false,
                                    gdk::EventMask::KEY_PRESS_MASK|
                                    gdk::EventMask::KEY_RELEASE_MASK,
                                    None,
                                    0,
                                );
                                if status != gdk::GrabStatus::Success {
                                    pointer.ungrab(0);
                                    return Err(Failure::new(FailureReason::KeyboardGrabFailed(status)));
                                }
                                return Ok(PointerAndKeyboard{
                                    pointer: pointer,
                                    keyboard: keyboard,
                                })
                            } else {
                                panic!("window not realized!!!")
                            }
                        } else {
                            return Err(Failure::new(FailureReason::KeyboardNotFound));
                        }
                    } else {
                        return Err(Failure::new(FailureReason::PointerNotFound));
                    }
                } else {
                    return Err(Failure::new(FailureReason::NoDeviceManager));
                }
            } else {
                return Err(Failure::new(FailureReason::NoDefaultScreen));
            };
        }

        fn ungrab(&self) {
            self.pointer.ungrab(0);
            self.keyboard.ungrab(0);
        }
    }

    pub fn is_available() -> bool {
        if cfg!(target_os = "windows") {
            false
        } else {
            SelectAreaDataCore::is_makeable() && PointerAndKeyboard::is_makeable()
        }
    }

    pub fn select_area() -> Result<gdk::Rectangle, Failure> {
        let sad = SelectAreaData::create()?;
        let pointer_and_keyboard = PointerAndKeyboard::grab(&sad.window)?;

        gtk::main();

        pointer_and_keyboard.ungrab();

        sad.finish()
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
