use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::FileLocation;
use crate::adapter::{chrono, crypto};
use crate::entity::processed_file::ProcessedFile;

pub struct FileLoader<'a> {
    pub locations: &'a FileLocation,
}

impl<'a> FileLoader<'a> {
    pub fn create_file(path: &PathBuf) {
        if !path.is_file() {
            if path.exists() {
                panic!("exist not a file: {path:?}");
            }
            File::create(path).unwrap().write_all(b"").unwrap();
        }
    }

    pub fn create_dir(path: &PathBuf) {
        if !path.is_dir() {
            if path.exists() {
                panic!("exist not a directory: {path:?}");
            }
            fs::create_dir(path).unwrap();
        }
    }

    pub fn load_stored_file(&self) -> (Vec<ProcessedFile>, HashSet<String>) {
        let index_string = fs::read_to_string(&self.locations.index_path).unwrap();

        let files: Vec<ProcessedFile> = index_string.lines()
            .map(|line| ProcessedFile::decode_stored_file(line, &self.locations.store_path))
            .collect();
        let names = files.iter()
            .fold(HashSet::<String>::new(), |mut name_set, file| {
                if let ProcessedFile::Stored { path, .. } = file {
                    let relative_name = path.strip_prefix(&self.locations.store_path).unwrap().to_str().unwrap().to_owned();
                    name_set.insert(relative_name);
                }
                name_set
            });
        (files, names)
    }

    pub fn save_stored_files(&self, stored_file_lines: Vec<String>) {
        let text = stored_file_lines.into_iter().reduce(|f, s| { f + "\n" + &s }).unwrap();
        let mut file = fs::OpenOptions::new().write(true).truncate(true).open(&self.locations.index_path).unwrap();
        file.write_all(text.as_bytes()).unwrap();
        file.flush().unwrap();
    }

    pub fn read_new_file(file_path: &PathBuf) -> ProcessedFile {
        let file = File::open(file_path).unwrap();
        let metadata = file.metadata().unwrap();
        ProcessedFile::New {
            hash: crypto::get_hash(&file),
            size: metadata.len(),
            path: file_path.to_owned(),
            year: chrono::year(metadata.modified().unwrap()),
        }
    }

    pub fn read_stored_file(file_path: &PathBuf) -> ProcessedFile {
        let file = File::open(file_path).unwrap();
        let metadata = file.metadata().unwrap();
        ProcessedFile::Stored {
            hash: crypto::get_hash(&file),
            size: metadata.len(),
            path: file_path.to_owned(),
        }
    }

    pub fn move_duplicate(&self, file: &ProcessedFile) {
        if let ProcessedFile::Duplicate { path, stored_path, duplicate_path, ln_path, .. } = file {
            if !duplicate_path.exists() && !ln_path.exists() {
                fs::rename(path, duplicate_path).unwrap();
                std::os::unix::fs::symlink(stored_path, ln_path).unwrap();
            }
        }
    }

    pub fn move_storage(&self, file: &ProcessedFile) {
        if let ProcessedFile::NewStored { path, stored_path, .. } = file {
            if !stored_path.exists() {
                fs::rename(path, stored_path).unwrap();
            }
        }
    }

    pub fn create_year_dir(&self, year: i32) {
        let year_storage_path = &self.locations.store_path.join(year.to_string());
        FileLoader::create_dir(year_storage_path);
    }

    pub fn find_files_recursive(dir: &PathBuf) -> Vec<PathBuf> {
        let mut result = Vec::new();
        for entry in fs::read_dir(dir).unwrap() {
            let entry_path = entry.unwrap().path();
            if entry_path.is_dir() {
                result.extend(FileLoader::find_files_recursive(&entry_path));
                continue;
            } else {
                result.push(entry_path);
            }
        }
        result
    }
}