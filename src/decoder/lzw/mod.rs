mod errors;
mod types;
use bitter::{BitReader, LittleEndianReader};
use errors::*;
use types::*;
use types::{Code, SpecialCode};
fn create_inverse_code_table(minimum_code_size: u8) -> InvCodeTable {
    use InvCode::*;
    use SpecialCode::*;
    let mut ret = InvCodeTable::new();
    for i in 0..((2 as u32).pow(minimum_code_size.into())) {
        // TO-DO: Return Error here instead?
        ret.push(CodeList(vec![Code::from(
            i as u16,
            minimum_code_size.into(),
        )
        .unwrap()]));
    }
    ret.push(ControlCode(ClearCodeInv));
    ret.push(ControlCode(EoiCodeInv));
    ret
}

fn lift_code_to_u8(codes: &Vec<Code>) -> Vec<&u8> {
    codes
        .iter()
        .map(|x| match x {
            Code::Entry(val) => val,
            // TO-DO: Return Error here instead?
            // But in theory, this should never happen
            // because of checks elsewhere in this codebase
            _ => {
                panic!("Code List should not contain a Special Code!")
            }
        })
        .collect()
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
    let get_code = |k, t: &InvCodeTable| match t.get(k as usize) {
        Some(val) => Some(val.clone()),
        None => None,
    };

    let mut index_stream: Vec<u8> = Vec::new();
    let mut code_stream = LittleEndianReader::new(&compressed_data);
    let code_key = code_stream.read_bits(cur_code_size).unwrap();
    let code = get_code(code_key, &inv_code_table);

    // Should always start with Clear Code Inventory
    assert_eq!(code, Some(InvCode::ControlCode(SpecialCode::ClearCodeInv)));

    let code_key = code_stream.read_bits(cur_code_size).unwrap();
    let code = get_code(code_key, &inv_code_table).unwrap();
    let mut prev_code_key = code_key;

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
        let code_key = code_stream.read_bits(cur_code_size).unwrap();
        let code = get_code(code_key, &inv_code_table);
        let k = match &code {
            Some(val) => match val {
                InvCode::CodeList(lst) => {
                    let lst: Vec<&u8> = lift_code_to_u8(&lst);
                    let k = lst[0];
                    index_stream.extend(lst);
                    *k
                }
                InvCode::ControlCode(special_code) => match special_code {
                    SpecialCode::ClearCodeInv => {
                        inv_code_table.clear();
                        inv_code_table.extend(create_inverse_code_table(minimum_code_size));
                        // I don't know why I need to re-declare it but I do...
                        let get_code = |k| match inv_code_table.get(k as usize) {
                            Some(val) => Some(val.clone()),
                            None => None,
                        };
                        cur_code_size = (minimum_code_size as u32) + 1;

                        // This code is also repeated from just before the for loop
                        // (Since it is going back to the beginning from after resetting
                        // the inverse code table). Can be lifted/extracted?
                        let code_key = code_stream.read_bits(cur_code_size).unwrap();
                        let code = get_code(code_key).unwrap();
                        prev_code_key = code_key;

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
                                continue;
                            }
                            // TO-DO: Return Error here instead?
                            _ => panic!("First value should be a Code List!"),
                        }
                    }
                    SpecialCode::EoiCodeInv => {
                        break;
                    }
                },
            },
            // Code not in inv_code_table
            None => {
                // Should be always safe to unwrap
                // Since it was previously checked
                match get_code(prev_code_key, &inv_code_table).unwrap() {
                    InvCode::CodeList(lst) => {
                        // TO-DO: Return Error here instead?
                        assert!(lst.len() >= 1);
                        // lst should not contain a special code
                        let lst: Vec<&u8> = lift_code_to_u8(&lst);
                        let k = lst[0];
                        index_stream.extend(lst);
                        index_stream.push(*k);
                        *k
                    }
                    // TO-DO: Return Error here instead?
                    _ => panic!("prev_code should not be a special code!"),
                }
            }
        };
        {
            // Should be always safe to unwrap
            // Since it was previously checked
            match get_code(prev_code_key, &inv_code_table).unwrap() {
                InvCode::CodeList(lst) => {
                    // TO-DO: Return Error here instead?
                    inv_code_table.push(InvCode::CodeList(
                        vec![lst, vec![Code::from(k as u16, minimum_code_size).unwrap()]].concat(),
                    ));
                }
                InvCode::ControlCode(_) => panic!("Previouc Code Should not be a Control Code!"),
            }
        }

        const MAX_CODE_SIZE: u32 = 12;
        if inv_code_table.len() == (2 as usize).pow(cur_code_size) && cur_code_size < MAX_CODE_SIZE
        {
            cur_code_size += 1;
        }
        prev_code_key = code_key;
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

    #[test]
    fn decompress_valid_stream() {
        let compressed_data: Vec<u8> = vec![
            140, 45, 153, 135, 42, 28, 220, 51, 160, 2, 117, 236, 149, 250, 168, 222, 96, 140, 4,
            145, 76, 1,
        ];
        let decompressed_data: Vec<u8> = vec![
            1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2,
            2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 1,
            1, 1, 2, 2, 2, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1,
            1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1,
        ];
        // TO-DO: Test with different min_code_size
        // and also make sure ClearCodeInv coverage
        // is done (it's not right now)
        assert_eq!(decompress(compressed_data, 2), Ok(decompressed_data));
    }
}
