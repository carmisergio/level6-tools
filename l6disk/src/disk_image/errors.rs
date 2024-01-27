use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum EncodeErrorType {
    SectorDivision,
    SectorNumber,
    TrackEncoding(&'static str),
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
        let string: String = match self.kind {
            EncodeErrorType::SectorDivision => format!("Unable to divide input image into sectors"),
            EncodeErrorType::SectorNumber => format!("Wrong number of sectors in input image"),
            EncodeErrorType::TrackEncoding(msg) => format!("Track encoding error: {}", msg),
        };

        write!(f, "{}", string)
    }
}

impl Error for EncodeError {}
