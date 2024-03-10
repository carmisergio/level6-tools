use core::fmt;
#[derive(Debug, Clone, PartialEq)]
pub enum Mnemonic {
    // Assembler directives
    DotORG,
    DotDB,
    DotDW,
    DotDD,
    DotDQ,

    // Branch on Registers instructions
    BL,
    BGE,
    BG,
    BLE,
    BOV,
    BNOV,
    BBT,
    BBF,
    BCT,
    BCF,
    BIOT,
    BIOF,
    BAL,
    BAGE,
    BE,
    BNE,
    BAG,
    BALE,
    BSU,
    BSE,
    B,

    // Branch on Indicators instructions
    BLZ,
    BGEZ,
    BEZ,
    BNEZ,
    BGZ,
    BLEZ,
    BODD,
    BEVN,
    BINC,
    BDEC,

    // Short Value Immediate instructions
    LDV,
    CMV,
    ADV,
    MLV,
}

impl Mnemonic {
    // Get StatementKind for each mnemonic
    pub fn get_kind(&self) -> StatementKind {
        match *self {
            // Assembler directives
            Self::DotORG => StatementKind::Org,
            Self::DotDB => StatementKind::DataDefinition,
            Self::DotDW => StatementKind::DataDefinition,
            Self::DotDD => StatementKind::DataDefinition,
            Self::DotDQ => StatementKind::DataDefinition,

            // Branch on Indicators instructions
            Self::BL => StatementKind::BranchOnIndicators,
            Self::BGE => StatementKind::BranchOnIndicators,
            Self::BG => StatementKind::BranchOnIndicators,
            Self::BLE => StatementKind::BranchOnIndicators,
            Self::BOV => StatementKind::BranchOnIndicators,
            Self::BNOV => StatementKind::BranchOnIndicators,
            Self::BBT => StatementKind::BranchOnIndicators,
            Self::BBF => StatementKind::BranchOnIndicators,
            Self::BCT => StatementKind::BranchOnIndicators,
            Self::BCF => StatementKind::BranchOnIndicators,
            Self::BIOT => StatementKind::BranchOnIndicators,
            Self::BIOF => StatementKind::BranchOnIndicators,
            Self::BAL => StatementKind::BranchOnIndicators,
            Self::BAGE => StatementKind::BranchOnIndicators,
            Self::BE => StatementKind::BranchOnIndicators,
            Self::BNE => StatementKind::BranchOnIndicators,
            Self::BAG => StatementKind::BranchOnIndicators,
            Self::BALE => StatementKind::BranchOnIndicators,
            Self::BSU => StatementKind::BranchOnIndicators,
            Self::BSE => StatementKind::BranchOnIndicators,
            Self::B => StatementKind::BranchOnIndicators,

            // Branch on Registers instructions
            Self::BLZ => StatementKind::BranchOnRegisters,
            Self::BGEZ => StatementKind::BranchOnRegisters,
            Self::BEZ => StatementKind::BranchOnRegisters,
            Self::BNEZ => StatementKind::BranchOnRegisters,
            Self::BGZ => StatementKind::BranchOnRegisters,
            Self::BLEZ => StatementKind::BranchOnRegisters,
            Self::BODD => StatementKind::BranchOnRegisters,
            Self::BEVN => StatementKind::BranchOnRegisters,
            Self::BINC => StatementKind::BranchOnRegisters,
            Self::BDEC => StatementKind::BranchOnRegisters,

            // Short Value Immediate instructions
            Self::LDV => StatementKind::ShortValueImmediate,
            Self::CMV => StatementKind::ShortValueImmediate,
            Self::ADV => StatementKind::ShortValueImmediate,
            Self::MLV => StatementKind::ShortValueImmediate,
        }
    }
}

impl Mnemonic {
    fn display_value(&self) -> &str {
        match *self {
            // Assembler directives
            Self::DotORG => ".ORG",
            Self::DotDB => ".DB",
            Self::DotDW => ".DW",
            Self::DotDD => ".DD",
            Self::DotDQ => ".DQ",

            // Branch on Indicators instructions
            Self::BL => "BL",
            Self::BGE => "BGE",
            Self::BG => "BG",
            Self::BLE => "BLE",
            Self::BOV => "BOV",
            Self::BNOV => "BNOV",
            Self::BBT => "BBT",
            Self::BBF => "BBF",
            Self::BCT => "BCT",
            Self::BCF => "BCF",
            Self::BIOT => "BIOT",
            Self::BIOF => "BIOF",
            Self::BAL => "BAL",
            Self::BAGE => "BAGE",
            Self::BE => "BE",
            Self::BNE => "BNE",
            Self::BAG => "BAG",
            Self::BALE => "BALE",
            Self::BSU => "BSU",
            Self::BSE => "BSE",
            Self::B => "B",
            // Branch on Registers instructions
            Self::BLZ => "BLZ",
            Self::BGEZ => "BGEZ",
            Self::BEZ => "BEZ",
            Self::BNEZ => "BNEZ",
            Self::BGZ => "BGZ",
            Self::BLEZ => "BLEZ",
            Self::BODD => "BODD",
            Self::BEVN => "BEVN",
            Self::BINC => "BINC",
            Self::BDEC => "BDEC",

            // Short Value Immediate instructions
            Self::LDV => "LDV",
            Self::CMV => "CMV",
            Self::ADV => "ADV",
            Self::MLV => "MLV",
        }
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_value())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Org,
    DataDefinition,
    BranchOnIndicators,
    BranchOnRegisters,
    ShortValueImmediate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Org(u64),
    DataDefinition(DataDefinitionSize, Vec<i128>),
    BranchOnIndicators(BranchOnIndicatorsOpCode, BranchLocation),
    BranchOnRegisters(BranchOnRegistersOpCode, DataRegister, BranchLocation),
    ShortValueImmediate(ShortValueImmediateOpCode, DataRegister, i128),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataDefinitionSize {
    Byte,
    Word,
    DoubleWord,
    QuadWord,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchOnIndicatorsOpCode {
    BL,
    BGE,
    BG,
    BLE,
    BOV,
    BNOV,
    BBT,
    BBF,
    BCT,
    BCF,
    BIOT,
    BIOF,
    BAL,
    BAGE,
    BE,
    BNE,
    BAG,
    BALE,
    BSU,
    BSE,
    NOP,
    B,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchOnRegistersOpCode {
    BLZ,
    BGEZ,
    BEZ,
    BNEZ,
    BGZ,
    BLEZ,
    BODD,
    BEVN,
    BINC,
    BDEC,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShortValueImmediateOpCode {
    LDV,
    CMV,
    ADV,
    MLV,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchLocation {
    Absolute(AddressExpression),
    LongDisplacement(AddressExpression),
    ShortDisplacement(AddressExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddressExpression {
    Immediate(u64),
    Label(String),
    WordDisplacement(i128),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataRegister {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}
