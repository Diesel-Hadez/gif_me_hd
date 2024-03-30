use std::fmt;

#[derive(PartialEq)]
pub enum CodeParseError {
    // first is the value
    // second is the minimum_code_size
    CodeTooBig(u16, u8),
    MinCodeSizeInvalid(u8),
}

impl fmt::Display for CodeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CodeParseError::{CodeTooBig, MinCodeSizeInvalid};
        match self {
            CodeTooBig(value, min_code_size) => {
                let max = (2 as u16).pow(*min_code_size as u32);
                write!(
                    f,
                    "LZW Code too big! Max is {} and entered is {}!",
                    max, value
                )
            }
            MinCodeSizeInvalid(min_code_size) => {
                write!(
                    f,
                    "LZW Min Code Size of {} is invalid! Only 2 to 8 inclusive is allowed!",
                    min_code_size
                )
            }
            _ => {
                // Should never reach here
                unimplemented!("Unknown CodeParseError occurred!");
            }
        }
    }
}
impl fmt::Debug for CodeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CodeParseError::{CodeTooBig, MinCodeSizeInvalid};
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())?;
        match self {
            CodeTooBig(value, min_code_size) => {
                let max = (2 as u16).pow(*min_code_size as u32);
                write!(
                    f,
                    "LZW Code too big! Max is {} and entered is {}!",
                    max, value
                )
            }
            MinCodeSizeInvalid(min_code_size) => {
                write!(
                    f,
                    "LZW Min Code Size of {} is invalid! Only 2 to 8 inclusive is allowed!",
                    min_code_size
                )
            }
            _ => {
                // Should never reach here
                unimplemented!("Unknown CodeParseError occurred!");
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DecompressError {
    KeyDoesNotExist,
}
