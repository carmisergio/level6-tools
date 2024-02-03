use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ConvertErrorType {
    SectorDivision,
    SectorNumber(usize, usize),
    DiskEncoding(String),
    RawImageCreation(String),
}

#[derive(Debug)]
pub struct ConvertError {
    kind: ConvertErrorType,
}

impl ConvertError {
    pub fn new(kind: ConvertErrorType) -> Self {
        Self { kind }
    }
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string: String = match &self.kind {
            ConvertErrorType::SectorDivision => {
                format!("Unable to divide input image into sectors")
            }
            ConvertErrorType::SectorNumber(should_be, is) => format!(
                "Wrong number of sectors in input image (should be {}, is {})",
                should_be, is
            ),
            ConvertErrorType::DiskEncoding(msg) => format!("Disk encoding error: {}", msg),
            ConvertErrorType::RawImageCreation(msg) => format!("Raw image creation error: {}", msg),
        };

        write!(f, "{}", string)
    }
}

impl Error for ConvertError {}
