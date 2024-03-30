use super::errors;
use errors::*;
#[derive(Debug, PartialEq, Clone)]
pub enum SpecialCode {
    ClearCodeInv,
    EoiCodeInv,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Code {
    // Max size of code table is 2**8+2
    // but at least 2 of them must be the ClearCodeInv
    // and EoiCodeInv
    // so it should fit in a u8
    Entry(u8),
    ControlCode(SpecialCode),
}

impl Code {
    pub fn from(value: u16, minimum_code_size: u8) -> Result<Self, CodeParseError> {
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
pub enum InvCode {
    CodeList(Vec<Code>),
    ControlCode(SpecialCode),
}

pub type InvCodeTable = Vec<InvCode>;
