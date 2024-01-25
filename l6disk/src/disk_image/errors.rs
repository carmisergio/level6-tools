use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum EncodeErrorType {
    SectorDivision,
}

#[derive(Debug)]
pub struct EncodeError {
    kind: EncodeErrorType,
}

impl EncodeError {
    pub fn new(kind: EncodeErrorType) -> Self {
        Self { kind }
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self.kind {
            EncodeErrorType::SectorDivision => "Unable to divide input image into sectors",
        };

        write!(f, "{}", string)
    }
}

impl Error for EncodeError {}
