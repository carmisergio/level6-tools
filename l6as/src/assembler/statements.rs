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

    // Single Operand instructions
    INC,
    DEC,
    NEG,
    CPL,
    CL,
    CLH,
    CMZ,
    CMN,
    CAD,
    STS,
    JMP,
    ENT,
    LEV,
    SAVE,
    RSTR,
    LB,
    LBF,
    LBT,
    LBC,
    LBS,
    AID,
    LDI,
    SDI,
    SID,

    // NoOp instruction
    NOP,

    // Generic instruction
    HLT,
    MCL,
    RTT,
    RTCN,
    RTCF,
    WDTN,
    WDTF,
    BRK,
    MMM,
    ASD,
    VLD,
    QOH,
    QOT,
    DQH,
    DQA,
    RSC,

    // Double Operand instructions
    LDR,
    STR,
    SRM,
    SWR,
    CMR,
    ADD,
    SUB,
    MUL,
    DIV,
    OR,
    XOR,
    AND,
    LDH,
    STH,
    CMH,
    ORH,
    XOH,
    ANH,
    LLH,
    MTM,
    STM,
    LDB,
    STB,
    CMB,
    SWB,
    LAB,
    LNJ,

    // Shift Short instructions
    SOL,
    SCL,
    SAL,
    DCL,
    SOR,
    SCR,
    SAR,
    DCR,

    // Shift Long instructions
    DOL,
    DAL,
    DOR,
    DAR,
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

            // Single Operand Instructions
            Self::INC => StatementKind::SingleOperandData,
            Self::DEC => StatementKind::SingleOperandData,
            Self::NEG => StatementKind::SingleOperandData,
            Self::CPL => StatementKind::SingleOperandData,
            Self::CL => StatementKind::SingleOperandData,
            Self::CLH => StatementKind::SingleOperandData,
            Self::CMZ => StatementKind::SingleOperandData,
            Self::CMN => StatementKind::SingleOperandBase,
            Self::CAD => StatementKind::SingleOperandData,
            Self::STS => StatementKind::SingleOperandData,
            Self::JMP => StatementKind::SingleOperandMemonly,
            Self::ENT => StatementKind::SingleOperandMemonly,
            Self::LEV => StatementKind::SingleOperandDataMasked,
            Self::SAVE => StatementKind::SingleOperandMemonlyMasked,
            Self::RSTR => StatementKind::SingleOperandMemonlyMasked,
            Self::LB => StatementKind::SingleOperandDataMasked,
            Self::LBF => StatementKind::SingleOperandDataMasked,
            Self::LBT => StatementKind::SingleOperandDataMasked,
            Self::LBC => StatementKind::SingleOperandDataMasked,
            Self::LBS => StatementKind::SingleOperandDataMasked,
            Self::AID => StatementKind::SingleOperandData,
            Self::LDI => StatementKind::SingleOperandData,
            Self::SDI => StatementKind::SingleOperandData,
            Self::SID => StatementKind::SingleOperandData,

            // NoOp instruction
            Self::NOP => StatementKind::NoOp,

            // Generic instruction
            Self::HLT => StatementKind::Generic,
            Self::MCL => StatementKind::Generic,
            Self::RTT => StatementKind::Generic,
            Self::RTCN => StatementKind::Generic,
            Self::RTCF => StatementKind::Generic,
            Self::WDTN => StatementKind::Generic,
            Self::WDTF => StatementKind::Generic,
            Self::BRK => StatementKind::Generic,
            Self::MMM => StatementKind::Generic,
            Self::ASD => StatementKind::Generic,
            Self::VLD => StatementKind::Generic,
            Self::QOH => StatementKind::Generic,
            Self::QOT => StatementKind::Generic,
            Self::DQH => StatementKind::Generic,
            Self::DQA => StatementKind::Generic,
            Self::RSC => StatementKind::Generic,

            // Double Operand instructions
            Self::LDR => StatementKind::DoubleOperandData,
            Self::STR => StatementKind::DoubleOperandData,
            Self::SRM => StatementKind::DoubleOperandDataMasked,
            Self::SWR => StatementKind::DoubleOperandData,
            Self::CMR => StatementKind::DoubleOperandData,
            Self::ADD => StatementKind::DoubleOperandData,
            Self::SUB => StatementKind::DoubleOperandData,
            Self::MUL => StatementKind::DoubleOperandData,
            Self::DIV => StatementKind::DoubleOperandData,
            Self::OR => StatementKind::DoubleOperandData,
            Self::XOR => StatementKind::DoubleOperandData,
            Self::AND => StatementKind::DoubleOperandData,
            Self::LDH => StatementKind::DoubleOperandData,
            Self::STH => StatementKind::DoubleOperandData,
            Self::CMH => StatementKind::DoubleOperandData,
            Self::ORH => StatementKind::DoubleOperandData,
            Self::XOH => StatementKind::DoubleOperandData,
            Self::ANH => StatementKind::DoubleOperandData,
            Self::LLH => StatementKind::DoubleOperandData,
            Self::MTM => StatementKind::DoubleOperandMode,
            Self::STM => StatementKind::DoubleOperandMode,
            Self::LDB => StatementKind::DoubleOperandData,
            Self::STB => StatementKind::DoubleOperandBase,
            Self::CMB => StatementKind::DoubleOperandBase,
            Self::SWB => StatementKind::DoubleOperandBase,
            Self::LAB => StatementKind::DoubleOperandNoreg,
            Self::LNJ => StatementKind::DoubleOperandMemonly,

            // Shift Short instructions
            Self::SOL => StatementKind::ShiftShort,
            Self::SCL => StatementKind::ShiftShort,
            Self::SAL => StatementKind::ShiftShort,
            Self::DCL => StatementKind::ShiftShort,
            Self::SOR => StatementKind::ShiftShort,
            Self::SCR => StatementKind::ShiftShort,
            Self::SAR => StatementKind::ShiftShort,
            Self::DCR => StatementKind::ShiftShort,

            // Shift Long instructions
            Self::DOL => StatementKind::ShiftLong,
            Self::DAL => StatementKind::ShiftLong,
            Self::DOR => StatementKind::ShiftLong,
            Self::DAR => StatementKind::ShiftLong,
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

            // Single Operand instructions
            Self::INC => "INC",
            Self::DEC => "DEC",
            Self::NEG => "NEG",
            Self::CPL => "CPL",
            Self::CL => "CL",
            Self::CLH => "CLH",
            Self::CMZ => "CMZ",
            Self::CMN => "CMN",
            Self::CAD => "CAD",
            Self::STS => "STS",
            Self::JMP => "JMP",
            Self::ENT => "ENT",
            Self::LEV => "LEV",
            Self::SAVE => "SAVE",
            Self::RSTR => "RSTR",
            Self::LB => "LB",
            Self::LBF => "LBF",
            Self::LBT => "LBT",
            Self::LBC => "LBC",
            Self::LBS => "LBS",
            Self::AID => "AID",
            Self::LDI => "LDI",
            Self::SDI => "SDI",
            Self::SID => "SID",

            // NoOp instruction
            Self::NOP => "NOP",

            // Generic instruction
            Self::HLT => "HLT",
            Self::MCL => "MCL",
            Self::RTT => "RTT",
            Self::RTCN => "RTCN",
            Self::RTCF => "RTCF",
            Self::WDTN => "WDTN",
            Self::WDTF => "WDTF",
            Self::BRK => "BRK",
            Self::MMM => "MMM",
            Self::ASD => "ASD",
            Self::VLD => "VLD",
            Self::QOH => "QOH",
            Self::QOT => "QOT",
            Self::DQH => "DQH",
            Self::DQA => "DQA",
            Self::RSC => "RSC",

            // Double Operand instructions
            Self::LDR => "LDR",
            Self::STR => "STR",
            Self::SRM => "SRM",
            Self::SWR => "SWR",
            Self::CMR => "CMR",
            Self::ADD => "ADD",
            Self::SUB => "SUB",
            Self::MUL => "MUL",
            Self::DIV => "DIV",
            Self::OR => "OR",
            Self::XOR => "XOR",
            Self::AND => "AND",
            Self::LDH => "LDH",
            Self::STH => "STH",
            Self::CMH => "CMH",
            Self::ORH => "ORH",
            Self::XOH => "XOH",
            Self::ANH => "ANH",
            Self::LLH => "LLH",
            Self::MTM => "MTM",
            Self::STM => "STM",
            Self::LDB => "LDB",
            Self::STB => "STB",
            Self::CMB => "CMB",
            Self::SWB => "SWB",
            Self::LAB => "LAB",
            Self::LNJ => "LNJ",

            // Shift Short instructions
            Self::SOL => "SOL",
            Self::SCL => "SCL",
            Self::SAL => "SAL",
            Self::DCL => "DCL",
            Self::SOR => "SOR",
            Self::SCR => "SCR",
            Self::SAR => "SAR",
            Self::DCR => "DCR",

            // Shift Long instructions
            Self::DOL => "DOL",
            Self::DAL => "DAL",
            Self::DOR => "DOR",
            Self::DAR => "DAR",
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
    NoOp,
    BranchOnRegisters,
    ShortValueImmediate,
    SingleOperandData,
    SingleOperandBase,
    SingleOperandMemonly,
    SingleOperandDataMasked,
    SingleOperandMemonlyMasked,
    Generic,
    DoubleOperandData,
    DoubleOperandDataMasked,
    DoubleOperandBase,
    DoubleOperandNoreg,
    DoubleOperandMemonly,
    DoubleOperandMode,
    ShiftShort,
    ShiftLong,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Org(u64),
    DataDefinition(DataDefinitionSize, Vec<i128>),
    BranchOnIndicators(BranchOnIndicatorsOpCode, BranchLocation),
    BranchOnRegisters(BranchOnRegistersOpCode, DataRegister, BranchLocation),
    ShortValueImmediate(ShortValueImmediateOpCode, DataRegister, i128),
    SingleOperand(SingleOperandOpCode, AddressSyllable, Option<i128>),
    DoubleOperand(DoubleOperandOpCode, Register, AddressSyllable, Option<i128>),
    Generic(GenericOpCode),
    ShiftShort(ShiftShortOpCode, DataRegister, u64),
    ShiftLong(ShiftLongOpCode, DataRegister, u64),
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
pub enum SingleOperandOpCode {
    INC,
    DEC,
    NEG,
    CPL,
    CL,
    CLH,
    CMZ,
    CMN,
    CAD,
    STS,
    JMP,
    ENT,
    LEV,
    SAVE,
    RSTR,
    LB,
    LBF,
    LBT,
    LBC,
    LBS,
    AID,
    LDI,
    SDI,
    SID,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenericOpCode {
    HLT,
    MCL,
    RTT,
    RTCN,
    RTCF,
    WDTN,
    WDTF,
    BRK,
    MMM,
    ASD,
    VLD,
    QOH,
    QOT,
    DQH,
    DQA,
    RSC,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DoubleOperandOpCode {
    LDR,
    STR,
    SRM,
    SWR,
    CMR,
    ADD,
    SUB,
    MUL,
    DIV,
    OR,
    XOR,
    AND,
    LDH,
    STH,
    CMH,
    ORH,
    XOH,
    ANH,
    LLH,
    MTM,
    STM,
    LDB,
    STB,
    CMB,
    SWB,
    LAB,
    LNJ,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShiftShortOpCode {
    SOL,
    SCL,
    SAL,
    DCL,
    SOR,
    SCR,
    SAR,
    DCR,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShiftLongOpCode {
    DOL,
    DAL,
    DOR,
    DAR,
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
pub enum AddressSyllable {
    RegisterAddressing(Register),
    ImmediateAddressing(ImmediateAddressMode),
    ImmediateOperand(i128),
    PRelative(PRelativeAddress),
    BRelative(BRelativeAddressMode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    Data(DataRegister),
    Base(BaseRegister),
    ModeControl(ModeControlRegister),
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

#[derive(Debug, Clone, PartialEq)]
pub enum BaseRegister {
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModeControlRegister {
    M1,
    M2,
    M3,
    M4,
    M5,
    M6,
    M7,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImmediateAddressMode {
    Direct(ImmediateAddress),
    Indirect(ImmediateAddress),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImmediateAddress {
    Simple(AddressExpression),
    Indexed(AddressExpression, DataRegister),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PRelativeAddress {
    Direct(AddressExpression),
    Indirect(AddressExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BRelativeAddressMode {
    Direct(BRelativeAddress),
    Indirect(BRelativeAddress),
    IncDecIndexed(BaseRegister, DataRegister, IncDec),
    PushPop(BaseRegister, IncDec),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BRelativeAddress {
    Simple(BaseRegister),
    Indexed(BaseRegister, DataRegister),
    Displacement(BaseRegister, i128),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IncDec {
    Increment,
    Decrement,
}
