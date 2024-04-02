use std::{fs::File, io::Read};
mod lzw;
mod parser;
mod types;
pub use parser::*;
pub use types::*;

pub fn load(filename: &str) -> Result<GifFile, &'static str> {
    match File::open(filename) {
        Ok(mut f) => {
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).expect("Unable to read file");
            GifFile::new(&buffer)
        }
        Err(_) => Err("Unable to load file"),
    }
}
