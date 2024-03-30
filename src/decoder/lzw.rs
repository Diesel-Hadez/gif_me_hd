use std::fmt;

use bitter::{BitReader, LittleEndianReader};

#[derive(Debug, PartialEq, Clone)]
enum SpecialCode {
    ClearCodeInv,
    EoiCodeInv,
}

#[derive(Debug, PartialEq, Clone)]
enum Code {
    // Max size of code table is 2**8+2
    // but at least 2 of them must be the ClearCodeInv
    // and EoiCodeInv
    // so it should fit in a u8
    Entry(u8),
    ControlCode(SpecialCode),
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
        use SpecialCode::*;
        if minimum_code_size < 2 || minimum_code_size > 8 {
            return Err(CodeParseError::MinCodeSizeInvalid(minimum_code_size));
        }
        let clear_code = (2 as u16).pow(minimum_code_size.into());
        let eoi_code = clear_code + 1;
        match value {
            x if x == clear_code => Ok(Code::ControlCode(ClearCodeInv)),
            x if x == eoi_code => Ok(Code::ControlCode(EoiCodeInv)),
            x if x > eoi_code => Err(CodeParseError::CodeTooBig(x, minimum_code_size)),
            _ => Ok(Code::Entry(value as u8)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum InvCode {
    CodeList(Vec<Code>),
    ControlCode(SpecialCode),
}

type InvCodeTable = Vec<InvCode>;

fn create_inverse_code_table(minimum_code_size: u8) -> InvCodeTable {
    use InvCode::*;
    use SpecialCode::*;
    let mut ret = InvCodeTable::new();
    for i in 0..((2 as u8).pow(minimum_code_size.into())) {
        ret.push(CodeList(vec![Code::Entry(i)]));
    }
    ret.push(ControlCode(ClearCodeInv));
    ret.push(ControlCode(EoiCodeInv));
    ret
}

#[derive(Debug, PartialEq)]
pub enum DecompressError {
    KeyDoesNotExist,
}

// Adapted from the python code (that I wrote myself) here
// https://github.com/GIF-ME-HD/gif_me_hd_proto/blob/master/gif_me_hd/lzw_gif3.py
pub fn decompress(
    compressed_data: Vec<u8>,
    minimum_code_size: u8,
) -> Result<Vec<u8>, DecompressError> {
    let mut inv_code_table = create_inverse_code_table(minimum_code_size);
    let mut cur_code_size: u32 = (minimum_code_size as u32) + 1;

    // Helper function to get a specific code from the code inv table
    // TO-DO Maybe error-handling here...
    let get_code = |k| match inv_code_table.get(k as usize) {
        Some(val) => Some(val.clone()),
        None => None,
    };

    let mut index_stream: Vec<u8> = Vec::new();
    let mut code_stream = LittleEndianReader::new(&compressed_data);
    let code_key = code_stream.read_bits(cur_code_size).unwrap();
    let code = get_code(code_key);

    // Should always start with Clear Code Inventory
    assert_eq!(code, Some(InvCode::ControlCode(SpecialCode::ClearCodeInv)));

    let code_key = code_stream.read_bits(cur_code_size).unwrap();

    let code = get_code(code_key).unwrap();
    let mut prev_code = code.clone();

    match code {
        InvCode::CodeList(lst) => {
            // This should always be an entry of size 1, for the first case.
            // TO-DO: Return Error here instead?
            assert_eq!(lst.len(), 1);
            index_stream.push(match lst[0] {
                Code::Entry(val) => val,
                // TO-DO: Return Error here instead?
                _ => panic!("First index should be an Entry!"),
            });
        }
        // TO-DO: Return Error here instead?
        _ => panic!("First value should be a Code List!"),
    }

    loop {
        // I don't know why I need to re-declare it but I do...
        let get_code = |k| match inv_code_table.get(k as usize) {
            Some(val) => Some(val.clone()),
            None => None,
        };
        let code_key = code_stream.read_bits(cur_code_size).unwrap();
        let code = get_code(code_key);
        match &code {
            Some(val) => match val {
                InvCode::CodeList(lst) => {}
                InvCode::ControlCode(special_code) => match special_code {
                    SpecialCode::ClearCodeInv => {}
                    SpecialCode::EoiCodeInv => {
                        break;
                    }
                },
            },
            // Code not in inv_code_table
            None => {
                match prev_code {
                    InvCode::CodeList(lst) => {
                        // TO-DO: Return Error here instead?
                        assert!(lst.len() >= 1);
                        // lst should not contain a special code
                        let lst: Vec<&u8> = lst
                            .iter()
                            .map(|x| match x {
                                Code::Entry(val) => val,
                                // TO-DO: Return Error here instead?
                                // But in theory, this should never happen
                                // because of checks elsewhere in this codebase
                                _ => {
                                    panic!("Previous Code List should not contain a Special Code!")
                                }
                            })
                            .collect();
                        let k = lst[0];
                        index_stream.extend(lst);
                        index_stream.push(*k);
                        k
                    }
                    // TO-DO: Return Error here instead?
                    _ => panic!("prev_code should not be a special code!"),
                };
            }
        };

        inv_code_table.push(InvCode::CodeList(vec![]));

        const MAX_CODE_SIZE: u32 = 12;
        if inv_code_table.len() == (2 as usize).pow(cur_code_size) && cur_code_size < MAX_CODE_SIZE
        {
            cur_code_size += 1;
        }
        prev_code = code.unwrap().clone();
    }

    Ok(index_stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_code() {
        use Code::*;
        use SpecialCode::*;
        assert_eq!(Code::from(0, 2), Ok(Entry(0)));
        assert_eq!(Code::from(3, 2), Ok(Entry(3)));
        assert_eq!(Code::from(1, 3), Ok(Entry(1)));
        assert_eq!(Code::from(7, 3), Ok(Entry(7)));
        assert_eq!(Code::from(8, 3), Ok(ControlCode(ClearCodeInv)));
        assert_eq!(Code::from(9, 3), Ok(ControlCode(EoiCodeInv)));
        assert_eq!(Code::from(257, 8), Ok(ControlCode(EoiCodeInv)));
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
