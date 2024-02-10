use super::preprocessor::PreprocessorParseErrorKind;
use std::path::PathBuf;

use colored::Colorize;

#[derive(Debug, Clone, PartialEq)]
pub enum PreprocessorWarningKind {
    GarbageAtEndOfLine(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreprocessorWarning {
    pub line_n: usize,
    pub file_name: PathBuf,
    pub line: String,
    pub kind: PreprocessorWarningKind,
}

impl PreprocessorWarning {
    pub fn message(&self) -> String {
        match &self.kind {
            PreprocessorWarningKind::GarbageAtEndOfLine(garbage) => {
                format!("unexpected garbage at end of line: \"{}\"", garbage)
            }
        }
    }
}

#[derive(Debug)]
pub struct PreprocessorError {
    pub line_n: usize,
    pub file_name: PathBuf,
    pub line: String,
    pub kind: PreprocessorParseErrorKind,
}

impl PreprocessorError {
    pub fn message(&self) -> String {
        match &self.kind {
            PreprocessorParseErrorKind::IncludeMissingFilePath => {
                format!("missing file path for %include")
            }
            PreprocessorParseErrorKind::DefineMissingIdentifier => {
                format!("missing identifier for %define")
            }
            PreprocessorParseErrorKind::DefineMissingValue(ident) => {
                format!("missing value for %define \"{}\"", ident)
            }
            PreprocessorParseErrorKind::MacroMissingIdentifier => {
                format!("missing identifier for %macro")
            }
            PreprocessorParseErrorKind::MacroMalformedArguments(ident) => {
                format!("malformed argument list for %macro \"{}\"", ident)
            }
            PreprocessorParseErrorKind::Nom => {
                format!("unknown parsing error")
            }
        }
    }
}

pub fn print_preprocessor_warning(msg: PreprocessorWarning) {
    println!(
        "{} [preprocessor]: {}",
        "warning".bright_yellow(),
        msg.message()
    );
    println!(
        "  --> {} {}{} {}",
        msg.file_name.display(),
        msg.line_n.to_string().bold(),
        "|".bright_blue(),
        msg.line
    );
}

pub fn print_preprocessor_error(msg: PreprocessorError) {
    println!("{} [preprocessor] {}", "error".bright_red(), msg.message());
    println!(
        "  --> {} {}{} {}",
        msg.file_name.display(),
        msg.line_n.to_string().bold(),
        "|".bright_blue(),
        msg.line
    );
}
