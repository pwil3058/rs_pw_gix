// Copyright 2024 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::collections::HashMap;
use std::fs;
use std::io::{self, Seek};
use std::path;

use fs2::FileExt;

type RecollectionDb = HashMap<String, String>;

pub struct Recollections {
    pub file_path: Option<path::PathBuf>,
}

impl Recollections {
    pub fn new(o_file_path: Option<&path::Path>) -> Recollections {
        if let Some(ref file_path) = o_file_path {
            if !file_path.exists() {
                if let Some(dir_path) = file_path.parent() {
                    if !dir_path.exists() {
                        fs::create_dir_all(dir_path).unwrap_or_else(|err| {
                            panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                        });
                    }
                }
                let mut file = fs::File::create(file_path)
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
                serde_json::to_writer(&mut file, &RecollectionDb::new())
                    .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            }
            Recollections {
                file_path: Some(file_path.to_path_buf()),
            }
        } else {
            Recollections { file_path: None }
        }
    }

    pub fn set_data_file_path(&mut self, file_path: &path::Path) {
        if !file_path.exists() {
            if let Some(dir_path) = file_path.parent() {
                if !dir_path.exists() {
                    fs::create_dir_all(dir_path).unwrap_or_else(|err| {
                        panic!("{:?}: line {:?}: {:?}", file!(), line!(), err)
                    });
                }
            }
            let mut file = fs::File::create(file_path)
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            serde_json::to_writer(&mut file, &RecollectionDb::new())
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
        };
        self.file_path = Some(file_path.to_path_buf())
    }

    pub fn recall(&self, name: &str) -> Option<String> {
        if let Some(ref file_path) = self.file_path {
            let file = fs::File::open(file_path)
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            file.lock_shared()
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            let hash_map: RecollectionDb = serde_json::from_reader(&file)
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            file.unlock()
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            hash_map.get(name).map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn recall_or_else(&self, name: &str, default: &str) -> String {
        match self.recall(name) {
            Some(string) => string,
            None => default.to_string(),
        }
    }

    pub fn remember(&self, name: &str, value: &str) {
        if let Some(ref file_path) = self.file_path {
            let mut file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(file_path)
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            file.lock_exclusive()
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            let mut hash_map: RecollectionDb = serde_json::from_reader(&file)
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            hash_map.insert(name.to_string(), value.to_string());
            file.seek(io::SeekFrom::Start(0))
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            file.set_len(0)
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            serde_json::to_writer(&mut file, &hash_map)
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
            file.unlock()
                .unwrap_or_else(|err| panic!("{:?}: line {:?}: {:?}", file!(), line!(), err));
        }
    }
}
