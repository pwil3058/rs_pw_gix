// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

//! Provide a mechanism for widgets to remember configuration
//! data (size, position, etc.) from one session to the next.

mod recollect;

//pub mod recollections {
use crate::recollect::*;
use lazy_static::*;
use mut_static::*;
use std::path;

lazy_static! {
    static ref RECOLLECTIONS: MutStatic<Recollections> = MutStatic::from(Recollections::new(None));
}

/// Initialise the mechanism by providing the path of the file
/// where the data should be stored.  This would normally be a
/// hidden file in the user's home directory or a hidden configuration
/// directory for the application.
///
/// This function should normally be called early in the application's
/// `main()` function e.g.
///
/// ```no_run
/// fn main_() {
///     use dirs;
///     use recollections;
///
///     let home_dir = dirs::home_dir().expect("badly designed OS");
///     recollections::init(&home_dir.join(".this_apps_recollections"));
/// }
/// ```
///
/// If this initialisation is not performed then calls to `recall()`
/// will return `None`, calls to `recall_or_else()` will return the
/// default supplied and calls to `remember()` will be ignored.
/// The operation of the application will not be effected otherwise.
pub fn init<P: AsRef<path::Path>>(file_path: P) {
    let file_path: &path::Path = file_path.as_ref();
    RECOLLECTIONS.write().unwrap().set_data_file_path(file_path);
}

/// Return the `String` value associated with the given `name` or
/// `None` if `pw_gix::recollections` has not been initialised or
/// asked remember data associated with the given `name`.
pub fn recall(name: &str) -> Option<String> {
    RECOLLECTIONS.read().unwrap().recall(name)
}

/// Return the `String` value associated with the given `name` or
/// `default` if `pw_gix::recollections` has not been initialised or
/// asked remember data associated with the given `name`.
pub fn recall_or_else(name: &str, default: &str) -> String {
    RECOLLECTIONS.read().unwrap().recall_or_else(name, default)
}

/// Remember the string specified by `value` and associate it with
/// the given `name` for later recall.
pub fn remember(name: &str, value: &str) {
    RECOLLECTIONS.read().unwrap().remember(name, value)
}
//}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path;

    #[test]
    fn recollect_test() {
        let recollection_file = path::Path::new(".recollection_test");
        let recollections = recollect::Recollections::new(Some(&recollection_file));
        assert_eq!(recollections.recall("anything"), None);
        assert_eq!(recollections.recall_or_else("anything", "but"), "but");
        recollections.remember("anything", "whatever");
        assert_eq!(
            recollections.recall("anything"),
            Some("whatever".to_string())
        );
        assert_eq!(recollections.recall_or_else("anything", "but"), "whatever");
        if let Err(err) = fs::remove_file(recollection_file) {
            panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
        }
    }
}
