extern crate cairo;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;

extern crate fs2;
#[macro_use]
extern crate lazy_static;
extern crate num;
extern crate mut_static;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
pub mod pwo {
    #[macro_export]
    macro_rules! implement_pwo {
        ( $f:ty, $field:ident, $t:ty ) => (
            impl PackableWidgetInterface for $f {
                type PackableWidgetType = $t;

                fn pwo(&self) -> $t {
                    self.$field.clone()
                }
            }
        )
    }

    extern crate glib;
    extern crate gtk;

    pub trait PackableWidgetInterface {
        type PackableWidgetType: glib::IsA<gtk::Widget>;

        fn pwo(&self) -> Self::PackableWidgetType;
    }
}

pub mod cairox;
pub mod colour;
pub mod dialogue;
pub mod gdkx;
pub mod gdk_pixbufx;
pub mod gtkx;
pub mod rgb_math;

pub mod recollect {
    use std::collections::HashMap;
    use std::fs;
    use std::io::{self, Seek};
    use std::path;

    use fs2::FileExt;
    use serde_json;

    type RDB = HashMap<String, String>;

    pub struct Recollections {
        pub file_path: Option<path::PathBuf>
    }

    impl Recollections {
        pub fn new(o_file_path: Option<&path::Path>) -> Recollections {
            if let Some(ref file_path) = o_file_path {
                if !file_path.exists() {
                    if let Some(dir_path) = file_path.parent() {
                        if !dir_path.exists() {
                            fs::create_dir_all(&dir_path).unwrap_or_else(
                                |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                            );
                        }
                    }
                    let file = fs::File::create(file_path).unwrap_or_else(
                        |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                    );
                    serde_json::to_writer(&file, &RDB::new()).unwrap_or_else(
                        |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                    );
                }
                Recollections{file_path: Some(file_path.to_path_buf())}
            } else {
                Recollections{file_path: None}
            }
        }

        pub fn set_data_file_path(&mut self, file_path: &path::Path) {
            if !file_path.exists() {
                if let Some(dir_path) = file_path.parent() {
                    if !dir_path.exists() {
                        fs::create_dir_all(&dir_path).unwrap_or_else(
                            |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                        );
                    }
                }
                let file = fs::File::create(file_path).unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                serde_json::to_writer(&file, &RDB::new()).unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
            };
            self.file_path = Some(file_path.to_path_buf())
        }

        pub fn recall(&self, name: &str) -> Option<String> {
            if let Some(ref file_path) = self.file_path {
                let file = fs::File::open(file_path).unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                file.lock_shared().unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                let hash_map: RDB = serde_json::from_reader(&file).unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                file.unlock().unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                match hash_map.get(name) {
                    Some(ref s) => Some(s.to_string()),
                    None => None
                }
            } else {
                None
            }
        }

        pub fn recall_or_else(&self, name: &str, default: &str) -> String {
            match self.recall(name) {
                Some(string) => string,
                None => default.to_string()
            }
        }

        pub fn remember(&self, name: &str, value: &str) {
            if let Some(ref file_path) = self.file_path {
                let mut file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(file_path).unwrap_or_else(
                        |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                    );
                file.lock_exclusive().unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                let mut hash_map: RDB = serde_json::from_reader(&file).unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                hash_map.insert(name.to_string(), value.to_string());
                file.seek(io::SeekFrom::Start(0)).unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                file.set_len(0).unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                serde_json::to_writer(&file, &hash_map).unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
                file.unlock().unwrap_or_else(
                    |err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                );
            }
        }
    }
}

pub mod recollections {
    use std::path;
    use recollect::*;
    use mut_static::*;

    lazy_static! {
        static ref RECOLLECTIONS: MutStatic<Recollections> = {
            MutStatic::from(Recollections::new(None))
        };
    }

    pub fn init(file_path: &path::Path) {
        RECOLLECTIONS.write().unwrap().set_data_file_path(file_path);
    }

    pub fn recall(name: &str) -> Option<String> {
        RECOLLECTIONS.read().unwrap().recall(name)
    }


    pub fn recall_or_else(name: &str, default: &str) -> String {
        RECOLLECTIONS.read().unwrap().recall_or_else(name, default)
    }

    pub fn remember(name: &str, value: &str) {
        RECOLLECTIONS.read().unwrap().remember(name, value)
    }
}

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
        assert_eq!(recollections.recall("anything"), Some("whatever".to_string()));
        assert_eq!(recollections.recall_or_else("anything", "but"), "whatever");
        if let Err(err) = fs::remove_file(recollection_file) {
            panic!("File: {:?} Line: {:?}: {:?}", file!(), line!(), err)
        }
    }
}
