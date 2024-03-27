use std::fmt;
pub struct ParseError;

impl fmt::Display for ParseError {
    pub fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An Error Occurred")
    }
}
impl fmt::Debug for ParseError {
    pub fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An Error Occurred")
    }
}
