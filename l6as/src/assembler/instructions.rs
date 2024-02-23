#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Org(u128),
    BranchOnIndicators(BranchOnIndicators),
    BranchOnRegisters(BranchOnRegisters),
}

#[derive(Debug, Clone, PartialEq)]
struct BranchOnIndicators {
    op: BranchOnIndicatorsOpCode,
    branchloc: BranchLocation,
}

#[derive(Debug, Clone, PartialEq)]
enum BranchOnIndicatorsOpCode {
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
}

#[derive(Debug, Clone, PartialEq)]
struct BranchOnRegisters {
    op: BranchOnRegistersOpCode,
    register: DataRegister,
    branchloc: BranchLocation,
}

#[derive(Debug, Clone, PartialEq)]
enum BranchOnRegistersOpCode {
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
enum BranchLocation {
    Absolute(Location),
    LongRelative(Location),
    ShortRelative(Location),
}

#[derive(Debug, Clone, PartialEq)]
enum Location {
    Immediate(u128),
    Label(String),
}

#[derive(Debug, Clone, PartialEq)]
enum DataRegister {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}
