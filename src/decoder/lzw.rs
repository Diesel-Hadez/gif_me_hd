use std::fmt;

use bitter::{BitReader, LittleEndianReader};

#[derive(Debug, PartialEq)]
enum Code {
    // Max size of code table is 2**8+2
    // but at least 2 of them must be the ClearCodeInv
    // and EoiCodeInv
    // so it should fit in a u8
    Entry(u8),
    ClearCodeInv,
    EoiCodeInv,
}

#[derive(PartialEq)]
enum CodeParseError {
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

impl Code {
    fn from(value: u16, minimum_code_size: u8) -> Result<Self, CodeParseError> {
        if minimum_code_size < 2 || minimum_code_size > 8 {
            return Err(CodeParseError::MinCodeSizeInvalid(minimum_code_size));
        }
        let clear_code = (2 as u16).pow(minimum_code_size.into());
        let eoi_code = clear_code + 1;
        match value {
            x if x == clear_code => Ok(Code::ClearCodeInv),
            x if x == eoi_code => Ok(Code::EoiCodeInv),
            x if x > eoi_code => Err(CodeParseError::CodeTooBig(x, minimum_code_size)),
            _ => Ok(Code::Entry(value as u8)),
        }
    }
}

fn create_inverse_code_table(minimum_code_size: u8) -> Vec<Code> {
    let mut ret = Vec::new();
    for i in 0..((2 as u8).pow(minimum_code_size.into())) {
        ret.push(Code::Entry(i));
    }
    ret.push(Code::ClearCodeInv);
    ret.push(Code::EoiCodeInv);
    ret
}

pub fn decompress(compressed_data: Vec<u8>, minimum_code_size: u8) -> Vec<u8> {
    let code_table: Vec<Code> = create_inverse_code_table(minimum_code_size);
    let mut next_smallest_code = ((2 as u8).pow(minimum_code_size.into())) + 2;
    let mut cur_idx = 1;
    let mut cur_code_size = minimum_code_size + 1;

    // Helper function since minimum_code_size
    // should stay the same
    let code_from = |c| Code::from(c as u16, minimum_code_size);

    let mut code_stream = LittleEndianReader::new(&compressed_data);
    let code = code_stream.read_bits(cur_code_size as u32).unwrap();
    let code = code_from(code);

    // Should always start with Clear Code Inventory
    assert_eq!(code, Ok(Code::ClearCodeInv));
    println!("First Code: {:#?}", code);
    compressed_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_code() {
        use Code::{ClearCodeInv, Entry, EoiCodeInv};
        assert_eq!(Code::from(0, 2), Ok(Entry(0)));
        assert_eq!(Code::from(3, 2), Ok(Entry(3)));
        assert_eq!(Code::from(1, 3), Ok(Entry(1)));
        assert_eq!(Code::from(7, 3), Ok(Entry(7)));
        assert_eq!(Code::from(8, 3), Ok(ClearCodeInv));
        assert_eq!(Code::from(9, 3), Ok(EoiCodeInv));
        assert_eq!(Code::from(257, 8), Ok(EoiCodeInv));
    }
    #[test]
    fn invalid_code() {
        assert_eq!(Code::from(6, 2), Err(CodeParseError::CodeTooBig(6, 2)));
        assert_eq!(Code::from(7, 2), Err(CodeParseError::CodeTooBig(7, 2)));
        assert_eq!(Code::from(10, 3), Err(CodeParseError::CodeTooBig(10, 3)));
        assert_eq!(Code::from(258, 8), Err(CodeParseError::CodeTooBig(258, 8)));
    }
    #[test]
    fn invalid_code_size() {
        assert_eq!(Code::from(0, 1), Err(CodeParseError::MinCodeSizeInvalid(1)));
        assert_eq!(Code::from(0, 9), Err(CodeParseError::MinCodeSizeInvalid(9)));
    }
}
