use super::statements::{BranchLocation, BranchOnIndicatorsOpCode, Statement};

/// Computes the size of a statement in memory (in words)
pub fn statement_size(statement: &Statement, _cur_addr: u64) -> u64 {
    // Compute different size depending on the kind of statement
    match statement {
        Statement::Org(_) => 0,
        Statement::BranchOnIndicators(op, branchloc) => {
            branch_on_indicators_inst_size(op, branchloc)
        }
    }
}

/// Computes the size of a Branch on Indicators instruction
pub fn branch_on_indicators_inst_size(
    _op: &BranchOnIndicatorsOpCode,
    branchloc: &BranchLocation,
) -> u64 {
    1 + branchloc_extra_words(branchloc)
}

/// Computes amount of extra words used by a Branch location
pub fn branchloc_extra_words(branchloc: &BranchLocation) -> u64 {
    match branchloc {
        BranchLocation::Absolute(_) => 1,
        BranchLocation::LongDisplacement(_) => 1,
        BranchLocation::ShortDisplacement(_) => 0,
    }
}
