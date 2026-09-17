// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::error;
use std::fmt;
use std::io;
use std::process::Command;

use crate::gdk::{self, prelude::WindowExtManual};
use crate::gdk_pixbuf::Pixbuf;
use crate::gtk;

use which::which;

#[derive(Debug)]
pub enum FailureReason {
    GrabFailed(gdk::GrabStatus),
    SeatNotFound,
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

impl Clone for FailureReason {
    // NB: this is necessary because io::Error doesn't implement copy OR clone
    fn clone(&self) -> FailureReason {
        match *self {
            FailureReason::GrabFailed(status) => FailureReason::GrabFailed(status),
            FailureReason::SeatNotFound => FailureReason::SeatNotFound,
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
                let description = error.to_string();
                let cloned_error = io::Error::new(kind, description);
                FailureReason::IOError(cloned_error)
            }
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
            FailureReason::GrabFailed(status) => format!("Grab failed: {}", status),
            FailureReason::SeatNotFound => "Seat not found".to_string(),
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
                let description = error.to_string();
                format!("I/O Error: {}", description)
            }
        };
        Failure { reason, message }
    }

    pub fn reason(&self) -> FailureReason {
        self.reason.clone()
    }

    pub fn user_cancelled(&self) -> bool {
        matches!(self.reason, FailureReason::UserCancelled)
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Failure {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<io::Error> for Failure {
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
        }
        Err(err) => {
            if !err.user_cancelled() {
                Command::new("gnome-screenshot").arg("-ac").spawn()?;
            }
        }
    };
    Ok(())
}

pub fn get_screen_pixbuf_rectangle(rect: &gdk::Rectangle) -> Option<Pixbuf> {
    let root_window = gdk::Window::default_root_window();
    root_window.pixbuf(rect.x(), rect.y(), rect.width(), rect.height())
}

pub mod area_selection {
    use super::{Failure, FailureReason};

    use std::cell::Cell;
    use std::convert::From;
    use std::rc::Rc;

    use crate::cairo;
    use crate::gdk::{self, SeatCapabilities};
    use crate::glib;
    use crate::gtk;
    use crate::gtk::prelude::*;

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct IntPoint {
        x: i32,
        y: i32,
    }

    impl From<(f64, f64)> for IntPoint {
        fn from(xy: (f64, f64)) -> IntPoint {
            IntPoint {
                x: xy.0 as i32,
                y: xy.1 as i32,
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct IntSize {
        width: i32,
        height: i32,
    }

    impl From<(IntPoint, IntPoint)> for IntSize {
        fn from(ipp: (IntPoint, IntPoint)) -> IntSize {
            IntSize {
                width: (ipp.0.x - ipp.1.x).abs(),
                height: (ipp.0.y - ipp.1.y).abs(),
            }
        }
    }

    fn top_left_corner(ipp: (IntPoint, IntPoint)) -> IntPoint {
        IntPoint {
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
            SelectAreaDataCore {
                start_position: Cell::new(None),
                current_position: Cell::new(None),
                end_position: Cell::new(None),
                button_num: Cell::new(None),
                window: gtk::Window::new(gtk::WindowType::Popup),
            }
        }

        fn is_makeable() -> bool {
            if let Some(screen) = gdk::Screen::default()
                && screen.is_composited()
            {
                return screen.rgba_visual().is_some();
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
            unsafe { self.window.destroy() };
            if let Some(start_position) = self.start_position.get()
                && let Some(end_position) = self.end_position.get()
            {
                let i_start: IntPoint = start_position.into();
                let i_last: IntPoint = end_position.into();
                let position = top_left_corner((i_start, i_last));
                let size = IntSize::from((i_start, i_last));
                return Ok(gdk::Rectangle::new(
                    position.x,
                    position.y,
                    size.width,
                    size.height,
                ));
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

            if let Some(screen) = gdk::Screen::default() {
                if screen.is_composited() {
                    if let Some(ref visual) = screen.rgba_visual() {
                        sad.window.set_visual(Some(visual));
                        sad.window.set_app_paintable(true);
                    } else {
                        return Err(Failure::new(FailureReason::NoRGBAVisual));
                    }
                } else {
                    return Err(Failure::new(FailureReason::NonCompositing));
                }
            } else {
                return Err(Failure::new(FailureReason::NoDefaultScreen));
            }
            let events = gdk::EventMask::KEY_PRESS_MASK
                | gdk::EventMask::BUTTON_PRESS_MASK
                | gdk::EventMask::BUTTON_RELEASE_MASK
                | gdk::EventMask::BUTTON_MOTION_MASK;
            sad.window.add_events(events);

            let sad_c = sad.clone();
            sad.window.connect_draw(move |_, cairo_context| {
                if let Some(start_position) = sad_c.start_position.get()
                    && let Some(current_position) = sad_c.current_position.get()
                {
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
                    cairo_context.set_operator(cairo::Operator::Xor);
                    cairo_context.stroke().expect("context stroke failed");
                };
                glib::Propagation::Proceed
                // gtk::Inhibit(false)
            });

            let sad_c = sad.clone();
            sad.window.connect_key_press_event(move |_, event| {
                if event.keyval() == gdk::keys::constants::Escape {
                    sad_c.start_position.set(None);
                    sad_c.current_position.set(None);
                    sad_c.end_position.set(None);
                    sad_c.button_num.set(None);

                    gtk::main_quit();
                }

                glib::Propagation::Stop
                // gtk::Inhibit(true)
            });

            let sad_c = sad.clone();
            sad.window.connect_button_press_event(move |_, event| {
                if !sad_c.in_progress() {
                    sad_c.button_num.set(Some(event.button()));
                    sad_c.start_position.set(Some(event.position()));
                }

                glib::Propagation::Stop
                // gtk::Inhibit(true)
            });

            let sad_c = sad.clone();
            sad.window
                .connect_button_release_event(move |window, event| {
                    if sad_c.in_progress_for(event.button()) {
                        sad_c.end_position.set(Some(event.position()));
                        sad_c.current_position.set(None);
                        sad_c.button_num.set(None);
                        window.queue_draw();

                        gtk::main_quit();
                    }

                    glib::Propagation::Stop
                    // gtk::Inhibit(true)
                });

            let sad_c = sad.clone();
            sad.window
                .connect_motion_notify_event(move |window, event| {
                    if sad_c.in_progress() {
                        sad_c.current_position.set(Some(event.position()));
                        window.queue_draw();
                    }

                    glib::Propagation::Stop
                    // gtk::Inhibit(true)
                });

            let root_window = gdk::Window::default_root_window();

            sad.window.move_(0, 0);
            sad.window.resize(root_window.width(), root_window.height());
            sad.window.show();

            Ok(sad)
        }
    }

    #[derive(Debug)]
    struct PointerAndKeyboard(gdk::Seat);

    impl PointerAndKeyboard {
        fn is_makeable() -> bool {
            if let Some(display) = gdk::Display::default()
                && let Some(seat) = display.default_seat()
            {
                return seat.pointer().is_some() && seat.keyboard().is_some();
            };
            false
        }

        fn grab<W: WidgetExt>(widget: &W) -> Result<PointerAndKeyboard, Failure> {
            let display =
                gdk::Display::default().ok_or(Failure::new(FailureReason::NoDefaultScreen))?;
            let seat = display
                .default_seat()
                .ok_or(Failure::new(FailureReason::SeatNotFound))?;
            let window = widget.window().expect("window not realized!");
            let _cursor = gdk::Cursor::for_display(&display, gdk::CursorType::Crosshair);
            let status = seat.grab(
                &window,
                SeatCapabilities::ALL,
                false,
                None,
                // Some(&cursor),
                None,
                None,
            );
            if status == gdk::GrabStatus::Success {
                Ok(PointerAndKeyboard(seat))
            } else {
                Err(Failure::new(FailureReason::GrabFailed(status)))
            }
        }

        fn ungrab(&self) {
            self.0.ungrab();
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
