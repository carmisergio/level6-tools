use super::preprocessor::LineLocation;
use std::{io, path::PathBuf};

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
pub enum PreprocessorErrorKind {
    // Lexer
    IncludeMissingFilePath,
    DefineMissingIdentifier,
    DefineMissingValue(String),

    // %include processing
    CannotOpenSourceFile(PathBuf),
    DobleInclusion(PathBuf),

    // %define processing
    DefineMultipleDefinition(String),
    DefineUndefined(String),

    // Unknown
    Nom(nom::error::ErrorKind),
}

#[derive(Debug)]
pub struct PreprocessorError {
    pub kind: PreprocessorErrorKind,
    pub location: Option<LineLocation>,
}

impl PreprocessorError {
    pub fn message(&self) -> String {
        match &self.kind {
            PreprocessorErrorKind::IncludeMissingFilePath => {
                format!("missing file path for %include")
            }
            PreprocessorErrorKind::DefineMissingIdentifier => {
                format!("missing identifier for %define")
            }
            PreprocessorErrorKind::DefineMissingValue(ident) => {
                format!("missing value for %define \"{}\"", ident)
            }
            PreprocessorErrorKind::CannotOpenSourceFile(file_path) => {
                format!("unable to find source file \"{}\": ", file_path.display())
            }
            PreprocessorErrorKind::DobleInclusion(file_path) => {
                format!("double %include for file \"{}\"", file_path.display())
            }
            PreprocessorErrorKind::DefineUndefined(identifier) => {
                format!("no %define for identifier \"{}\"", identifier)
            }
            PreprocessorErrorKind::DefineMultipleDefinition(identifier) => {
                format!("multiple %define for identifier \"{}\"", identifier)
            }
            PreprocessorErrorKind::Nom(kind) => {
                format!("unknown nom error: {:?}", kind)
            }
        }
    }
}

#[derive(Debug)]
pub enum AssemblerErrorKind {
    // Unknown
    Nom(nom::error::ErrorKind),
}

#[derive(Debug)]
pub struct AssemblerError {
    pub kind: AssemblerErrorKind,
    pub location: Option<LineLocation>,
}

impl AssemblerError {
    pub fn message(&self) -> String {
        match &self.kind {
            &AssemblerErrorKind::Nom(kind) => {
                format!("unknown nom error: {:?}", kind)
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
    println!("{} [preprocessor] {}", "error".bright_red(), err.message());

    if let Some(location) = err.location {
        println!(
            "  --> {} {}{} {}",
            location.file_name.file_name().unwrap().to_str().unwrap(),
            location.line_n.to_string().bold(),
            "|".bright_blue(),
            location.raw_content.trim()
        );
    }
}

pub fn print_assembler_error(err: AssemblerError) {
    println!("{} [assembler] {}", "error".bright_red(), err.message());

    if let Some(location) = err.location {
        println!(
            "  --> {} {}{} {}",
            location.file_name.file_name().unwrap().to_str().unwrap(),
            location.line_n.to_string().bold(),
            "|".bright_blue(),
            location.raw_content.trim()
        );
    }
}

pub fn print_final_error_msg() {
    println!(
        "l6as: {} encountered during processing, no output generated",
        "errors".bright_red()
    )
}

pub fn print_write_file_error_msg(err: io::Error) {
    println!(
        "{}: Unable to write output file: {}",
        "error".bright_red(),
        err
    );
}
