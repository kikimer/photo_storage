use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::constants::{HASH_LEN, HEX_CHARS_IN_BYTE, HEX_RADIX};

pub type FileHash = [u8; HASH_LEN];
pub type FileStorage = HashMap::<FileHashKey, Vec<ProcessedFile>>;

#[derive(Eq, PartialEq, Hash)]
pub struct FileHashKey {
    pub hash: FileHash,
    pub size: u64,
}

#[derive(Debug)]
pub enum ProcessedFile {
    Stored {
        hash: FileHash,
        size: u64,
        path: PathBuf,
    },
    NewStored {
        hash: FileHash,
        size: u64,
        path: PathBuf,
        stored_path: PathBuf,
        year: i32,
    },
    New {
        hash: FileHash,
        size: u64,
        path: PathBuf,
        year: i32,
    },
    Duplicate {
        path: PathBuf,
        stored_path: PathBuf,
        duplicate_path: PathBuf,
        ln_path: PathBuf,
    },
}

impl ProcessedFile {
    pub fn decode_stored_file(text: &str, base_path: &Path) -> ProcessedFile {
        let values: Vec<&str> = text.split('|').collect();
        let path = base_path.join(values[2]);
        ProcessedFile::Stored {
            hash: ProcessedFile::decode_hex(values[0]),
            size: u64::from_str(values[1]).unwrap(),
            path,
        }
    }

    pub fn encode_stored_file(&self, base_path: &PathBuf) -> String {
        let (hash, size, path) = match self {
            ProcessedFile::Stored { hash, size, path } => (hash, size, path),
            ProcessedFile::NewStored { hash, size, stored_path, .. } => (hash, size, stored_path),
            _ => panic!("can't save Processed file")
        };
        let mut text = String::new();
        text.push_str(&ProcessedFile::encode_hex(&hash[..]));
        text.push('|');
        text.push_str(&size.to_string());
        text.push('|');
        text.push_str(path.strip_prefix(base_path).unwrap().to_str().unwrap());
        text
    }

    fn decode_hex(s: &str) -> [u8; HASH_LEN] {
        let vector: Vec<u8> = (0..HASH_LEN * HEX_CHARS_IN_BYTE)
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..=i + 1], HEX_RADIX).unwrap())
            .collect();
        vector.try_into().unwrap()
    }

    fn encode_hex(hex: &[u8]) -> String {
        hex.iter().map(|value| { format!("{:02x?}", value) }).collect()
    }
}
