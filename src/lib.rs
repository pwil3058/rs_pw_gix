extern crate gdk;
extern crate gtk;

extern crate fs2;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod gdkx;
pub mod gtkx;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
