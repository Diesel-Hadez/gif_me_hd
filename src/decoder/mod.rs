use std::{fs::File, io::Read};
mod parser;
mod types;
mod lzw;
use parser::*;
use types::*;

pub fn load(filename: &str) -> Result<GifFile, &'static str> {
    match File::open(filename) {
        Ok(mut f) => {
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).expect("Unable to read file");
            GifFile::new(&buffer)
        }
        Err(_) => {
            Err("Unable to load file")
        }
    }
}
