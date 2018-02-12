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

use gdk;

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
    which("gnome-screenshot").is_ok()
}

pub fn take_screen_sample() -> Result<(), Failure> {
    Command::new("gnome-screenshot").arg("-ac").spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
