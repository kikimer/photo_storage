use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::adapter::file_loader::FileLoader;
use crate::unique_relative_path;

pub struct FileLocation {
    pub store_path: PathBuf,
    pub duplicates_path: PathBuf,
    pub new_files_path: PathBuf,
    pub index_path: PathBuf,
}

impl FileLocation {
    pub fn new(root: &Path) -> FileLocation {
        FileLocation {
            store_path: root.join("store"),
            duplicates_path: root.join("duplicates"),
            new_files_path: root.join("new_files"),
            index_path: root.join("index.txt"),
        }
    }

    pub fn create_locations(&self) {
        FileLoader::create_file(&self.index_path);
        FileLoader::create_dir(&self.store_path);
        FileLoader::create_dir(&self.duplicates_path);
        FileLoader::create_dir(&self.new_files_path);
    }

    pub fn new_files(&self) -> Vec<PathBuf> {
        FileLoader::find_files_recursive(&self.new_files_path)
    }

    pub fn get_duplicates_filenames(&self) -> HashSet<String> {
        let mut duplicate_names = HashSet::new();
        FileLoader::find_files_recursive(&self.duplicates_path).into_iter()
            .for_each(|file| {
                let file_name = file.file_name().unwrap().to_str().unwrap();
                unique_relative_path(&PathBuf::from(""), file_name, &mut duplicate_names);
            });
        duplicate_names
    }
}