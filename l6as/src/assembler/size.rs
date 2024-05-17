use super::statements::{AddressSyllable, BranchLocation, DataDefinitionSize, Statement};

/// Computes the size of a statement in memory (in words)
pub fn statement_size(statement: &Statement, _cur_addr: u64) -> u64 {
    // Compute different size depending on the kind of statement
    match statement {
        Statement::Org(_) => 0,
        Statement::DataDefinition(size, chunks) => data_definition_dir_size(size, chunks),
        Statement::BranchOnIndicators(_op, branchloc) => branch_inst_size(branchloc),
        Statement::BranchOnRegisters(_op, _reg, branchloc) => branch_inst_size(branchloc),
        Statement::ShortValueImmediate(_op, _reg, _value) => 1,
        Statement::SingleOperand(_op, addr_syl, mask) => single_operand_inst_size(addr_syl, mask),
        Statement::Generic(_op) => 1,
        Statement::DoubleOperand(_op, _reg, addr_syl, mask) => {
            double_operand_inst_size(addr_syl, mask)
        }
        Statement::ShiftShort(_op, _reg, _val) => 1,
        Statement::ShiftLong(_op, _reg, _val) => 1,
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

/// Computes the size of a Single Operand instruction
pub fn single_operand_inst_size(addr_syl: &AddressSyllable, mask: &Option<i128>) -> u64 {
    1 + address_syl_extra_words(addr_syl) + mask_extra_words(mask)
}

/// Computes the size of a Double Operand instruction
pub fn double_operand_inst_size(addr_syl: &AddressSyllable, mask: &Option<i128>) -> u64 {
    1 + address_syl_extra_words(addr_syl) + mask_extra_words(mask)
}

/// Computes amount of extra words used by a Branch location
pub fn branchloc_extra_words(branchloc: &BranchLocation) -> u64 {
    match branchloc {
        BranchLocation::Absolute(_) => 1,
        BranchLocation::LongDisplacement(_) => 1,
        BranchLocation::ShortDisplacement(_) => 0,
    }
}

/// Computes the size of a Branch on Indicators instruction
pub fn branch_inst_size(branchloc: &BranchLocation) -> u64 {
    1 + branchloc_extra_words(branchloc)
}

/// Computes amount of extra words used by an Address Syllable
pub fn address_syl_extra_words(address_syllable: &AddressSyllable) -> u64 {
    match address_syllable {
        AddressSyllable::RegisterAddressing(_) => 0,
        AddressSyllable::ImmediateAddressing(_) => 1,
        AddressSyllable::ImmediateOperand(_) => 1,
        AddressSyllable::BRelative(_) => 0,
        AddressSyllable::PRelative(_) => 1,
    }
}

/// Computes amount of extra words used by an Address Syllable
pub fn mask_extra_words(mask: &Option<i128>) -> u64 {
    match mask {
        None => 0,
        Some(_) => 1,
    }
}
