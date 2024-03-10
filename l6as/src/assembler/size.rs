use super::statements::{BranchLocation, DataDefinitionSize, Statement};

/// Computes the size of a statement in memory (in words)
pub fn statement_size(statement: &Statement, _cur_addr: u64) -> u64 {
    // Compute different size depending on the kind of statement
    match statement {
        Statement::Org(_) => 0,
        Statement::DataDefinition(size, chunks) => data_definition_dir_size(size, chunks),
        Statement::BranchOnIndicators(_op, branchloc) => branch_inst_size(branchloc),
        Statement::BranchOnRegisters(_op, _reg, branchloc) => branch_inst_size(branchloc),
        Statement::ShortValueImmediate(_op, _reg, _value) => 1,
    }
}

/// Computes words used by a data definition directive
pub fn data_definition_dir_size(size: &DataDefinitionSize, chunks: &Vec<i128>) -> u64 {
    let n_chunks = chunks.len() as u64;
    match size {
        DataDefinitionSize::Byte => (n_chunks + 1) / 2,
        DataDefinitionSize::Word => n_chunks,
        DataDefinitionSize::DoubleWord => n_chunks * 2,
        DataDefinitionSize::QuadWord => n_chunks * 4,
    }
}

/// Computes the size of a Branch on Indicators instruction
pub fn branch_inst_size(branchloc: &BranchLocation) -> u64 {
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
