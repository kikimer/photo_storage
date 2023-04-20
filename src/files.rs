use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use ring::digest;
use ring::digest::SHA256_OUTPUT_LEN;

pub struct FileInfo {
    pub relative_name: String,
    pub size: u64,
}

pub struct Settings {
    pub store_path: PathBuf,
    pub duplicates_path: PathBuf,
    pub new_files_path: PathBuf,
    pub index_path: PathBuf,
    pub index: HashMap<[u8; SHA256_OUTPUT_LEN], FileInfo>,
}

impl Settings {
    pub fn new(root: &Path) -> Settings {
        let index_path = file_path_from(root, "index.txt");
        let index = Settings::load_index(&index_path);

        Settings {
            store_path: dir_path_from(root, "store"),
            duplicates_path: dir_path_from(root, "duplicates"),
            new_files_path: dir_path_from(root, "new_files"),
            index_path,
            index,
        }
    }

    pub fn save_index(&self) {
        let mut text = String::new();
        for (hash, file_info) in &self.index {
            text.push_str(&encode_hex(hash));
            text.push('|');
            text.push_str(&file_info.size.to_string());
            text.push('|');
            text.push_str(&file_info.relative_name);
            text.push('\n');
        }
        let mut file = fs::OpenOptions::new().write(true).truncate(true).open(&self.index_path).unwrap();
        file.write_all(text.as_bytes()).unwrap();
        file.flush().unwrap();
    }

    fn load_index(index_path: &PathBuf) -> HashMap<[u8; SHA256_OUTPUT_LEN], FileInfo> {
        let index_string = fs::read_to_string(index_path).unwrap();
        let index_lines = index_string.lines();
        let mut index = HashMap::new();

        for record in index_lines {
            let values: Vec<&str> = record.split("|").collect();
            let key = decode_hex(&values[0]);
            let size: u64 = u64::from_str(&values[1]).unwrap();
            let relative_name = values[2].to_owned();

            index.insert(key, FileInfo { relative_name, size });
        }

        return index;
    }
}

fn decode_hex(s: &str) -> [u8; SHA256_OUTPUT_LEN] {
    let vector: Vec<u8> = (0..SHA256_OUTPUT_LEN * 2)
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..=i + 1], 16).unwrap())
        .collect();
    vector
        .try_into()
        .unwrap()
}

fn encode_hex(hex: &[u8]) -> String {
    hex.iter().map(|value| { format!("{:02x?}", value) }).collect()
}

pub fn get_hash(file: &File) -> [u8; SHA256_OUTPUT_LEN] {
    let mut context = digest::Context::new(&digest::SHA256);
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 1024];
    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    return context.finish().as_ref().try_into().unwrap();
}

pub fn dir_path_from(root: &Path, file_name: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    path.push(file_name);
    create_dir(&path);
    return path;
}

pub fn file_path_from(root: &Path, file_name: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    path.push(file_name);
    create_file(&path);
    return path;
}

fn create_file(path: &PathBuf) {
    if !path.is_file() {
        File::create(path).unwrap().write(b"").unwrap();
    }
}

pub fn create_dir(path: &PathBuf) {
    if !path.is_dir() {
        fs::create_dir(path).unwrap();
    }
}