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

// Concept of using indexes in a vec! as pointers taken from
// https://dev.to/deciduously/no-more-tears-no-more-knots-arena-allocated-trees-in-rust-44k6
// So that I don't have to worry about ownership and usage of smart pointers

#[derive(Debug)]
struct Node<T>
where
    T: PartialEq
{
    idx: usize,
    val: T,
    children: Vec<usize>,
}

impl<T> Node<T> 
where
    T: PartialEq
{
    fn new(idx: usize, val: T) -> Self {
        Self {
            idx,
            val,
            children: vec![],
        }
    }
}

#[derive(Debug, Default)]
struct CodeInvTree {
    values: Vec<Node<Code>>,
    root_children: Vec<usize>,
}

#[derive(PartialEq)]
enum TreeError {
    PathNotFound,
    NoPathSpecified,
}

impl CodeInvTree {
    fn insert(&mut self, val: Code) -> usize {
        let idx = self.values.len();
        self.values.push(Node::new(idx, val));
        idx
    }
    fn insert_root(&mut self, val: Code) -> usize {
        let idx = self.insert(val);
        self.root_children.push(idx);
        idx
    }

    fn insert_at(&mut self, path: &[Code], val: Code) -> Result<usize, TreeError> {
        let parent = self.find_path(path)?;
        let ret = self.insert(val);
        self.values[parent].children.push(ret);
        Ok(ret)
    }

    fn code_exists(&self, target: &Code) -> bool {
        match self.values
                    .iter()
                    .find(|&x| x.val == *target) {
                        Some(idx) => true,
                        None => false,
                    }
    }

    fn path_exists(&self, path: &[Code]) -> bool {
        match self.find_path(path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn find_path(&self, path: &[Code]) -> Result<usize, TreeError> {
        // No Path to nothingness
        if path.len() == 0 {
            return Err(TreeError::NoPathSpecified);
        }

        let mut cur_idx = 0;
        // Look in all root children
        cur_idx = match self.root_children
            .iter()
            .find(|&&x| self.values[x].val == path[0]) {
                Some(idx) => *idx,
                None => {
                    return Err(TreeError::PathNotFound);
                },
            };

        // Look for the rest
        for code in &path[1..] {
            cur_idx = match self.values[cur_idx]
            .children
            .iter()
            .find(|&&x| self.values[x].val == path[0]) {
                Some(idx) => *idx,
                None => {
                    return Err(TreeError::PathNotFound);
                },
            };
        }
        Ok(cur_idx)
    }
}

fn create_inverse_code_table(minimum_code_size: u8) -> CodeInvTree {
    let mut ret = CodeInvTree::default();
    for i in 0..((2 as u8).pow(minimum_code_size.into())) {
        ret.insert_root(Code::Entry(i));
    }
    ret.insert_root(Code::ClearCodeInv);
    ret.insert_root(Code::EoiCodeInv);
    ret
}

// Adapted from the python code (that I wrote myself) here
// https://github.com/GIF-ME-HD/gif_me_hd_proto/blob/master/gif_me_hd/lzw_gif3.py
pub fn decompress(compressed_data: Vec<u8>, minimum_code_size: u8) -> Vec<u8> {
    let code_table: CodeInvTree = create_inverse_code_table(minimum_code_size);
    let mut cur_idx = 1;
    let mut cur_code_size: u32 = (minimum_code_size as u32) + 1;

    // Helper function since minimum_code_size
    // should stay the same
    let code_from = |c| Code::from(c as u16, minimum_code_size);

    let mut index_stream: Vec<u8> = Vec::new();
    let mut code_stream = LittleEndianReader::new(&compressed_data);
    let code = code_stream.read_bits(cur_code_size).unwrap();
    let code = code_from(code);

    // Should always start with Clear Code Inventory
    assert_eq!(code, Ok(Code::ClearCodeInv));

    let code = code_stream.read_bits(cur_code_size).unwrap();
    let code = code_from(code).unwrap();

    // First one should always be an entry
    match code {
        Code::Entry(code_table_index) => {
            // This should be an entry too
            // TO-DO: Add Code here...
        }
        _ => panic!("First value should be an Entry Code!"),
    }

    let mut prev_code = code;
    loop {
        let code = code_stream.read_bits(cur_code_size).unwrap();
        let code = code_from(code).unwrap();
        let mut k: usize;

        // Note: If I used a hashmap, it would be a very 
        // inefficient at O(n) complexity where n is the number
        // of elements in the code_table, which is why I used a tree
        // which I think would be a better structure.
        if code_table.code_exists(&code) {
            match code {
                Code::Entry(val) => index_stream.push(val),
                Code::EoiCodeInv => break,
                Code::ClearCodeInv => {
                    // Need to reset code_table
                },
            }
        }
        else {
            // TO-DO
        }

        let next_smallest_code = code_table.values.len();

        // TO-DO: Push to code Table

        const MAX_CODE_SIZE: u32 = 12;
        if next_smallest_code == (2 as usize).pow(cur_code_size) - 1 && 
            cur_code_size < MAX_CODE_SIZE {
                cur_code_size += 1;
        }

        prev_code = code;
    }

    index_stream
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
