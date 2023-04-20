use std::fs;
use std::fs::{create_dir, File};
use std::path::{Path, PathBuf};
use std::str;

use chrono::{Datelike, DateTime, Local};

use crate::files::{FileInfo, Settings};

mod files;

pub fn run(root: &Path) -> Result<&'static str, &'static str> {
    let mut settings = Settings::new(root);
    let path = settings.new_files_path.to_owned();
    compute_dir(&mut settings, &path);
    settings.save_index();
    return Ok("");
}


fn compute_dir(settings: &mut Settings, path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry_path = entry.unwrap().path();
        if entry_path.is_dir() {
            compute_dir(settings, &entry_path);
            continue;
        }
        let file = File::open(&entry_path).unwrap();
        let file_size = file.metadata().unwrap().len();
        let hash = files::get_hash(&file);
        match settings.index.get(&hash) {
            Some(file_info) if file_info.size == file_size => {
                move_to_duplicates(settings, &entry_path, &file_info)
            }
            _ => {
                let year = get_creation_year(&file);
                create_year_dir(settings, &year);
                let mut store_folder = settings.store_path.to_owned();
                store_folder.push(&year);
                let store_file = unique_filename(&store_folder, entry_path.file_name().unwrap().to_str().unwrap());
                let relative_name = year.to_owned() + "/" + store_file.file_name().unwrap().to_str().unwrap();
                fs::rename(&entry_path, &store_file).unwrap();
                settings.index.insert(hash, FileInfo { relative_name, size: file_size });
            }
        }
    }
}

fn get_creation_year(file: &File) -> String {
    let creation_time: DateTime<Local> = file.metadata().unwrap().modified().unwrap().into();
    creation_time.year().to_string()
}

fn create_year_dir(settings: &Settings, year: &String) {
    let mut storage_path = settings.store_path.to_owned();
    storage_path.push(year);
    if !storage_path.is_dir() {
        create_dir(storage_path).unwrap();
    }
}

fn move_to_duplicates(settings: &Settings, path: &Path, file_storage: &FileInfo) {
    let dub_path = unique_filename(&settings.duplicates_path, path.file_name().unwrap().to_str().unwrap());
    fs::rename(path, &dub_path).unwrap();

    let ln_filename = dub_path.file_name().unwrap().to_str().unwrap().to_owned() + "_ln";
    let ln_path = unique_filename(&settings.duplicates_path, &ln_filename);

    let mut store_path = settings.store_path.to_owned();
    store_path.push(&file_storage.relative_name);
    std::os::unix::fs::symlink(store_path, ln_path).unwrap();
}

fn unique_filename(path: &PathBuf, filename: &str) -> PathBuf {
    let mut result = path.to_path_buf();
    result.push(filename);
    let (prefix_filename, suffix_filename) = if let Some(dot) = filename.rfind(".") {
        filename.split_at(dot)
    } else {
        (filename, "")
    };
    let mut index = 1;
    while result.exists() {
        let mut new_file_name = prefix_filename.to_owned();
        new_file_name.push('_');
        new_file_name.push_str(&index.to_string());
        new_file_name.push_str(&suffix_filename);
        result.set_file_name(new_file_name);
        index += 1;
    }
    return result;
}









