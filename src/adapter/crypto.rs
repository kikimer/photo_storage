use std::io::{BufReader, Read};

use ring::digest;

use crate::constants::HASH_LEN;

pub fn get_hash<T: Read>(reader: T) -> [u8; HASH_LEN] {
    let mut context = digest::Context::new(&digest::SHA256);
    let mut buf_reader = BufReader::new(reader);
    let mut buffer = [0u8; 1024];
    loop {
        let count = buf_reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    return context.finish().as_ref().try_into().unwrap();
}

#[cfg(test)]
mod tests {
    use ring::digest::SHA256_OUTPUT_LEN;

    use crate::constants::HASH_LEN;

    #[test]
    fn hash_size() {
        assert_eq!(HASH_LEN, SHA256_OUTPUT_LEN)
    }

    #[test]
    fn get_hash(){

    }
}