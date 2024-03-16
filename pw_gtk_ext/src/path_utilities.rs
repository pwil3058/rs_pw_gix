// Copyright 2024 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::env;
use std::ffi::OsString;
use std::fs::{DirEntry, FileType, Metadata, ReadDir};
use std::io;
use std::path::{Component, Path, PathBuf};

use log;

pub fn absolute_pathbuf(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else if path.starts_with("~/") {
        let home_dir_path = dirs::home_dir()?;
        if let Ok(tail) = path.strip_prefix("~/") {
            Some(home_dir_path.join(tail))
        } else {
            None
        }
    } else if let Ok(curr_dir) = env::current_dir() {
        let mut components = path.components();
        if let Some(first_component) = components.next() {
            if let Component::CurDir = first_component {
                Some(curr_dir.join(components.as_path()))
            } else {
                Some(curr_dir.join(path))
            }
        } else {
            Some(curr_dir)
        }
    } else {
        None
    }
}

#[derive(Debug)]
pub struct UsableDirEntry {
    dir_entry: DirEntry,
    file_type: FileType,
}

impl UsableDirEntry {
    pub fn path(&self) -> PathBuf {
        self.dir_entry.path()
    }

    pub fn file_name(&self) -> OsString {
        self.dir_entry.file_name()
    }

    pub fn is_dir(&self) -> bool {
        self.file_type.is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }

    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub fn metadata(&self) -> io::Result<Metadata> {
        self.dir_entry.metadata()
    }
}

pub struct UsableDirEntries {
    read_dir: ReadDir,
}

impl Iterator for UsableDirEntries {
    type Item = UsableDirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(result) = self.read_dir.next() {
            match result {
                Ok(dir_entry) => match dir_entry.metadata() {
                    Ok(metadata) => {
                        let file_type = metadata.file_type();
                        return Some(UsableDirEntry {
                            dir_entry,
                            file_type,
                        });
                    }
                    Err(err) => match err.kind() {
                        io::ErrorKind::NotFound => {
                            // We assume that "not found" is due to race condition and ignore it
                        }
                        io::ErrorKind::PermissionDenied => {
                            // benign so just log it in case someone cares
                            log::info!(
                                "{:?}: permission denied accessing metadata",
                                dir_entry.path()
                            )
                        }
                        _ => log::warn!(
                            "{:?}: unexpected error \"{err}\" accessing metadata",
                            dir_entry.path()
                        ),
                    },
                },
                Err(err) => match err.kind() {
                    io::ErrorKind::NotFound => {
                        // We assume that "not found" is due to race condition and ignore it
                    }
                    io::ErrorKind::PermissionDenied => {
                        // benign so just log it in case someone cares
                        log::info!("Permission denied for ReadDir;;next()")
                    }
                    _ => log::warn!("Unexpected error \"{err}\"  for ReadDir;;next()"),
                },
            }
        }
        None
    }
}

pub fn usable_dir_entries(dir_path: &Path) -> io::Result<UsableDirEntries> {
    let read_dir = dir_path.read_dir()?;
    Ok(UsableDirEntries { read_dir })
}
