use bitter::{BitReader, LittleEndianReader};

#[derive(Debug, PartialEq, Eq)]
enum Code {
    // Max size of code table is 2**8+2
    // but at least 2 of them must be the ClearCodeInv
    // and EoiCodeInv
    // so it should fit in a u8
    Entry(u8),
    ClearCodeInv,
    EoiCodeInv,
}

impl Code {
    fn from(value: u8, minimum_code_size: u8) -> Self {
        let clear_code = (2 as u8).pow(minimum_code_size.into());
        let eoi_code = (2 as u8).pow(minimum_code_size.into()) + 1;
        if value == clear_code {
            return Code::ClearCodeInv;
        } else if value == eoi_code {
            return Code::EoiCodeInv;
        }
        Code::Entry(value)
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
    let code_from = |c| Code::from(c as u8, minimum_code_size);

    let mut code_stream = LittleEndianReader::new(&compressed_data);
    let code = code_stream.read_bits(cur_code_size as u32).unwrap();
    let code = code_from(code);

    // Should always start with Clear Code Inventory
    assert_eq!(code, Code::ClearCodeInv);
    println!("First Code: {:#?}", code);
    compressed_data
}
