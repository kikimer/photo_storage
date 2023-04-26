use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::str;

use entity::file_location::FileLocation;

use crate::adapter::file_loader::FileLoader;
use crate::entity::processed_file::{FileHashKey, FileStorage, ProcessedFile};

mod adapter;
mod entity;
mod constants;

pub fn run(root: &Path, command: String) -> Result<(), &'static str> {
    if command.is_empty() {
        add_new_files(root)
    } else if command == "check" {
        check_stored_files(root)
    } else if command == "reindex" {
        rebuild_index(root)
    } else {
        Err("bad command")
    }
}

fn rebuild_index(root: &Path) -> Result<(), &'static str> {
    let locations = FileLocation::new(root);
    locations.create_locations();
    let file_loader = FileLoader { locations: &locations };
    let files_text = locations.stored_files()
        .into_iter()
        .map(|file_path| FileLoader::read_stored_file(&file_path)
            .encode_stored_file(&locations.store_path)
        )
        .collect();
    file_loader.save_stored_files(files_text);
    Ok(())
}

fn check_stored_files(root: &Path) -> Result<(), &'static str> {
    let locations = FileLocation::new(root);
    locations.create_locations();
    let file_loader = FileLoader { locations: &locations };

    let (mut files, _) = file_loader.load_stored_file();
    let check_files_iterator = locations.stored_files()
        .into_iter()
        .map(|file_path| FileLoader::read_new_file(&file_path));
    files.extend(check_files_iterator);
    files.into_iter()
        .fold(FileStorage::new(), group_by_hash)
        .into_values()
        .filter(|files| !is_correct_files(files))
        .for_each(|files| {
            eprintln!("{files:?}\n");
        });
    Ok(())
}

fn is_correct_files(files: &Vec<ProcessedFile>) -> bool {
    files.len() == 2
}

fn add_new_files(root: &Path) -> Result<(), &'static str> {
    let locations = FileLocation::new(root);
    locations.create_locations();
    let file_loader = FileLoader { locations: &locations };

    let new_files_iterator = locations.new_files()
        .into_iter()
        .map(|file_path| FileLoader::read_new_file(&file_path));

    let (mut files, mut stored_names) = file_loader.load_stored_file();
    let mut duplicate_names = locations.get_duplicates_filenames();
    files.extend(new_files_iterator.into_iter());

    let processed_files: Vec<ProcessedFile> = files.into_iter()
        .fold(FileStorage::new(), group_by_hash)
        .into_values()
        .map(|files| mark_new_storage(files, &mut stored_names, &locations))
        .flat_map(|files| mark_duplicates(files, &mut duplicate_names, &locations))
        .collect();

    processed_files.iter()
        .fold(HashSet::<i32>::new(), |mut years, file| {
            if let ProcessedFile::NewStored { year, .. } = file { years.insert(*year); }
            years
        }).iter()
        .for_each(|&year| {
            file_loader.create_year_dir(year);
        });

    processed_files.iter()
        .for_each(|file| {
            file_loader.move_storage(file);
            file_loader.move_duplicate(file);
        });

    let processed_files_text = processed_files.iter()
        .filter(|file| matches!(file, ProcessedFile::NewStored{..}) || matches!(file, ProcessedFile::Stored{..}))
        .map(|file| file.encode_stored_file(&locations.store_path))
        .collect();
    file_loader.save_stored_files(processed_files_text);

    Ok(())
}

fn mark_new_storage(mut grouped_files: Vec<ProcessedFile>, names: &mut HashSet<String>, locations: &FileLocation) -> Vec<ProcessedFile> {
    let has_stored_file = grouped_files.iter()
        .any(|file| matches!(file, ProcessedFile::Stored{..}));
    if !has_stored_file {
        if let ProcessedFile::New { hash, size, path, year } = grouped_files.pop().unwrap() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let year_path = PathBuf::from(year.to_string());
            let file_relative_path = unique_relative_path(&year_path, file_name, names);
            let stored_path = locations.store_path.join(file_relative_path);
            grouped_files.push(
                ProcessedFile::NewStored { hash, size, path, stored_path, year }
            )
        }
    }
    grouped_files
}

fn mark_duplicates(grouped_files: Vec<ProcessedFile>, duplicate_names: &mut HashSet<String>, locations: &FileLocation) -> Vec<ProcessedFile> {
    let stored_file = grouped_files.iter().find(|file| matches!(file, ProcessedFile::Stored {..}) || matches!(file, ProcessedFile::NewStored {..})).unwrap();
    let stored_path = match stored_file {
        ProcessedFile::Stored { path, .. } => path.to_owned(),
        ProcessedFile::NewStored { path, .. } => path.to_owned(),
        _ => panic!("no storage file for: {grouped_files:?}")
    };

    grouped_files.into_iter()
        .map(|file| {
            if let ProcessedFile::New { path, .. } = file {
                let duplicate_name = path.file_name().unwrap().to_str().unwrap();
                let duplicate_relative_path = unique_relative_path(&PathBuf::from(""), duplicate_name, duplicate_names);
                let ln_name = duplicate_relative_path.file_name().unwrap().to_str().unwrap().to_owned() + ".lnk";
                let ln_relative_path = unique_relative_path(&PathBuf::from(""), &ln_name, duplicate_names);
                ProcessedFile::Duplicate {
                    path,
                    stored_path: stored_path.to_owned(),
                    duplicate_path: locations.duplicates_path.join(duplicate_relative_path),
                    ln_path: locations.duplicates_path.join(ln_relative_path),
                }
            } else {
                file
            }
        }).collect()
}

fn unique_relative_path(relative_base_path: &Path, file_name: &str, names: &mut HashSet<String>) -> PathBuf {
    let (prefix, suffix) = if let Some(dot_index) = file_name.rfind('.') {
        file_name.split_at(dot_index)
    } else {
        (file_name, "")
    };

    let mut test_path = relative_base_path.join(file_name);
    let mut i = 0;
    while names.contains(test_path.to_str().unwrap()) {
        i += 1;
        test_path = relative_base_path.join(prefix.to_owned() + "_" + &i.to_string() + suffix);
    }

    names.insert(test_path.to_str().unwrap().to_owned());
    test_path
}

fn group_by_hash(mut file_index: FileStorage, processed_file: ProcessedFile) -> FileStorage {
    let key = match processed_file {
        ProcessedFile::Duplicate { .. } => { return file_index; }
        ProcessedFile::Stored { hash, size, .. } => FileHashKey { hash, size },
        ProcessedFile::New { hash, size, .. } => FileHashKey { hash, size },
        ProcessedFile::NewStored { hash, size, .. } => FileHashKey { hash, size },
    };

    let entry = file_index.get_mut(&key);
    match entry {
        Some(v) => { v.push(processed_file); }
        None => { file_index.insert(key, vec![processed_file]); }
    }
    file_index
}