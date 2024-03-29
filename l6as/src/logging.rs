use super::assembler::Mnemonic;
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
    Unknown,
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
            PreprocessorErrorKind::Unknown => {
                format!("unkown error")
            }
        }
    }
}

#[derive(Debug)]
pub enum AssemblerErrorKind {
    // Unknown
    Nom(nom::error::ErrorKind),

    // Labels
    LabelDoubleDefinition(String),

    // Statement parsing
    MnemonicRequired,
    UnkownMnemonic(String),

    // Argument parsing
    MalformedArgumentList,
    WrongNumberOfArguments(Mnemonic, usize, usize),
    InvalidAddress(String),
    UnexpectedCharactersAtEndOfArgument(String),
    InvalidBranchLocation(String),
    InvalidDataRegister(String),
    InvalidImmediateValue(String),
    InvalidDataDefinitionChunk(String),
    InvalidAddressSyllable(String),

    // Code Generation
    BranchAddressOutOfRange(u64),
    BranchLongDisplacementOutOfRange(i128),
    BranchShortDisplacementOutOfRange(i128),
    BranchShortDisplacementMustNotBe0Or1,
    WordDisplacementOutOfRange(i128),
    ShortImmediateValueOutOfRange(i128),
    UndefinedLabel(String),
    DataDefinitionValueOutOfRange(i128),
}

#[derive(Debug)]
pub struct AssemblerError {
    pub kind: AssemblerErrorKind,
    pub location: Option<LineLocation>,
}

impl AssemblerError {
    pub fn message(&self) -> String {
        match &self.kind {
            AssemblerErrorKind::Nom(kind) => {
                format!("unknown nom error: {:?}", kind)
            }
            AssemblerErrorKind::LabelDoubleDefinition(label) => {
                format!("Double definition for label: \"{}\"", label)
            }
            AssemblerErrorKind::MnemonicRequired => {
                format!("a mnemonic is required")
            }
            AssemblerErrorKind::UnkownMnemonic(mnemo) => {
                format!("unkown mnemonic: \"{}\"", mnemo)
            }
            AssemblerErrorKind::MalformedArgumentList => {
                format!("malformed argument list")
            }
            AssemblerErrorKind::WrongNumberOfArguments(mnemonic, expected, got) => {
                format!("{} takes {} arguments, got {}", mnemonic, expected, got)
            }
            AssemblerErrorKind::InvalidAddress(arg) => {
                format!("invalid address: \"{}\"", arg)
            }
            AssemblerErrorKind::InvalidBranchLocation(arg) => {
                format!("invalid branch location: \"{}\"", arg)
            }
            AssemblerErrorKind::InvalidDataRegister(arg) => {
                format!("invalid data register: \"{}\"", arg)
            }
            AssemblerErrorKind::InvalidImmediateValue(arg) => {
                format!("invalid immediate value: \"{}\"", arg)
            }
            AssemblerErrorKind::InvalidDataDefinitionChunk(arg) => {
                format!("invalid definition chunk: \"{}\"", arg)
            }
            AssemblerErrorKind::InvalidAddressSyllable(arg) => {
                format!("invalid address syllable: \"{}\"", arg)
            }
            AssemblerErrorKind::UnexpectedCharactersAtEndOfArgument(arg) => {
                format!("unexpected characters at end of argument: \"{}\"", arg)
            }
            AssemblerErrorKind::BranchAddressOutOfRange(addr) => {
                format!("address out of range: {:#X}", addr)
            }
            AssemblerErrorKind::BranchLongDisplacementOutOfRange(addr) => {
                format!("long displacement out of range: {}", addr)
            }
            AssemblerErrorKind::BranchShortDisplacementOutOfRange(addr) => {
                format!("short displacement out of range: {}", addr)
            }
            AssemblerErrorKind::BranchShortDisplacementMustNotBe0Or1 => {
                format!("short displacement must not be 0 or 1")
            }
            AssemblerErrorKind::WordDisplacementOutOfRange(addr) => {
                format!("word displacement out of range: {}", addr)
            }
            AssemblerErrorKind::ShortImmediateValueOutOfRange(val) => {
                format!("short immediate value out of range: ({:#X}) {}", val, val)
            }
            AssemblerErrorKind::UndefinedLabel(label) => {
                format!("undefined label: \"{}\"", label)
            }
            AssemblerErrorKind::DataDefinitionValueOutOfRange(val) => {
                format!("data definition value out of range: ({:#X}) {}", val, val)
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
