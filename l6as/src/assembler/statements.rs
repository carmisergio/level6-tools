use core::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Mnemonic {
    // Assembler directives
    DotORG,

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
}

impl Mnemonic {
    // Get StatementKind for each mnemonic
    pub fn get_kind(&self) -> StatementKind {
        match *self {
            // Assembler directives
            Self::DotORG => StatementKind::Org,

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
        }
    }
}

impl Mnemonic {
    fn display_value(&self) -> &str {
        match *self {
            // Assembler directives
            Self::DotORG => ".ORG",

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
    BranchOnIndicators,
    BranchOnRegisters,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Org(u64),
    BranchOnIndicators(BranchOnIndicatorsOpCode, BranchLocation),
    BranchOnRegisters(BranchOnRegistersOpCode, DataRegister, BranchLocation),
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
pub struct BranchOnRegisters {
    op: BranchOnRegistersOpCode,
    register: DataRegister,
    branchloc: BranchLocation,
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
