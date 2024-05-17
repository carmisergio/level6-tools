use super::assembler::{BaseRegister, DataRegister, Mnemonic};
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
    InvalidBaseRegister(String),
    InvalidModeControlRegister(String),
    InvalidImmediateValue(String),
    InvalidDataDefinitionChunk(String),
    InvalidAddressSyllable(String),
    WrongRegisterType(String, Mnemonic),
    RegisterAddressingInvalid(Mnemonic),
    ImmediateAddressingInvalid(Mnemonic),
    InvalidMaskWord(String),

    // Code Generation
    BranchAddressOutOfRange(u64),
    BranchLongDisplacementOutOfRange(i128),
    BranchShortDisplacementOutOfRange(i128),
    BranchShortDisplacementMustNotBe0Or1,
    WordDisplacementOutOfRange(i128),
    ShortImmediateValueOutOfRange(i128),
    UndefinedLabel(String),
    DataDefinitionValueOutOfRange(i128),
    ImmediateValueOutOfRange(i128),
    InvalidIndexRegister(DataRegister),
    ImmediateAddressOutOfRange(u64),
    DisplacementOutOfRange(i128),
    InvalidBaseRegisterAddrSyl(BaseRegister),
    MaskWordOutOfRange(i128),
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
            AssemblerErrorKind::InvalidBaseRegister(arg) => {
                format!("invalid register: \"{}\"", arg)
            }
            AssemblerErrorKind::InvalidModeControlRegister(arg) => {
                format!("invalid base register: \"{}\"", arg)
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
            AssemblerErrorKind::WrongRegisterType(arg, mnemo) => {
                format!("wrong register type for {}: \"{}\"", mnemo, arg)
            }
            AssemblerErrorKind::RegisterAddressingInvalid(mnemo) => {
                format!("register addressing invalid for {}", mnemo)
            }
            AssemblerErrorKind::ImmediateAddressingInvalid(mnemo) => {
                format!("immediate addressing invalid for {}", mnemo)
            }
            AssemblerErrorKind::InvalidMaskWord(arg) => {
                format!("invalid mask word: {}", arg)
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
            AssemblerErrorKind::ImmediateValueOutOfRange(val) => {
                format!("immediate value out of range: ({:#X}) {}", val, val)
            }
            AssemblerErrorKind::InvalidIndexRegister(reg) => {
                format!(
                    "invalid index register: {} ",
                    get_data_register_display_value(reg)
                )
            }
            AssemblerErrorKind::ImmediateAddressOutOfRange(addr) => {
                format!("immediate address out of range: ({:#X}) {}", addr, addr)
            }
            AssemblerErrorKind::DisplacementOutOfRange(disp) => {
                format!("displacement out of range: {}", disp)
            }
            AssemblerErrorKind::InvalidBaseRegisterAddrSyl(reg) => {
                format!(
                    "invalid base register: {}",
                    get_base_register_display_value(reg)
                )
            }
            AssemblerErrorKind::MaskWordOutOfRange(mask) => {
                format!("mask word out of range: ({:#X}) {}", mask, mask)
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

fn get_data_register_display_value(reg: &DataRegister) -> &str {
    match reg {
        DataRegister::R1 => "$R1",
        DataRegister::R2 => "$R2",
        DataRegister::R3 => "$R3",
        DataRegister::R4 => "$R4",
        DataRegister::R5 => "$R5",
        DataRegister::R6 => "$R6",
        DataRegister::R7 => "$R7",
    }
}

fn get_base_register_display_value(reg: &BaseRegister) -> &str {
    match reg {
        BaseRegister::B1 => "$B1",
        BaseRegister::B2 => "$B2",
        BaseRegister::B3 => "$B3",
        BaseRegister::B4 => "$B4",
        BaseRegister::B5 => "$B5",
        BaseRegister::B6 => "$B6",
        BaseRegister::B7 => "$B7",
    }
}
