use super::file::FileInclusionError;
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
pub enum PreprocessorError {
    Parser(PreprocessorParseErrorBody),
    FileInclusion(FileInclusionError),
}

#[derive(Debug)]
pub struct PreprocessorParseErrorBody {
    pub line_n: usize,
    pub file_name: PathBuf,
    pub line: String,
    pub kind: PreprocessorParseErrorKind,
}

impl PreprocessorParseErrorBody {
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
        msg.file_name.file_name().unwrap().to_str().unwrap(),
        msg.line_n.to_string().bold(),
        "|".bright_blue(),
        msg.line
    );
}

pub fn print_preprocessor_error(err: PreprocessorError) {
    match err {
        PreprocessorError::Parser(err) => print_preprocessor_parse_error(err),
        PreprocessorError::FileInclusion(err) => print_preprocessor_file_inclusion_error(err),
    }
}

pub fn print_preprocessor_parse_error(err: PreprocessorParseErrorBody) {
    println!("{} [preprocessor] {}", "error".bright_red(), err.message());
    println!(
        "  --> {} {}{} {}",
        err.file_name.file_name().unwrap().to_str().unwrap(),
        err.line_n.to_string().bold(),
        "|".bright_blue(),
        err.line
    );
}

pub fn print_preprocessor_file_inclusion_error(err: FileInclusionError) {
    match err {
        FileInclusionError::FileNotFound(file_path) => {
            println!(
                "{} [preprocessor] unable to open source file: \"{}\"",
                "error".bright_red(),
                file_path.display()
            );
        }
        FileInclusionError::DoubleInclusion(file_path) => {
            println!(
                "{} [preprocessor] double include for file: \"{}\"",
                "error".bright_red(),
                file_path.display()
            );
        }
    }
}

pub fn print_final_error_msg() {
    println!(
        "l6as: {} encountered during processing, no output generated",
        "errors".bright_red()
    )
}
