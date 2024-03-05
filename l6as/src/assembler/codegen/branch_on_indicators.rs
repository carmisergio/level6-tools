use std::collections::HashMap;

use crate::logging::AssemblerErrorKind;

use bit_struct::*;

use crate::assembler::statements::{BranchLocation, BranchOnIndicatorsOpCode};

use super::common::get_branch_location_field_value;

bit_struct! {
    pub struct BranchOnIndicatorsInstructionWord(u16) {
        header: u4,
        op: u5,
        branchloc: i7
    }
}

/// Generaete code for a Branch on Indicators instruction
pub fn codegen_branch_on_indicators(
    op: &BranchOnIndicatorsOpCode,
    branchloc: &BranchLocation,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Generate branch location field
    let (branchloc_field, mut extra_words) =
        get_branch_location_field_value(branchloc, cur_addr, label_table)?;

    // Build instruction word
    let inst_word = BranchOnIndicatorsInstructionWord::new(
        u4!(0b0000),
        get_branch_on_indicators_op_value(op),
        branchloc_field,
    );

    // Concatenate words
    let mut result = vec![inst_word.raw()];
    result.append(&mut extra_words); // Add potential extra word from branchloc field

    Ok(result)
}
fn get_branch_on_indicators_op_value(op: &BranchOnIndicatorsOpCode) -> u5 {
    match op {
        BranchOnIndicatorsOpCode::BL => u5!(0b00100),
        BranchOnIndicatorsOpCode::BGE => u5!(0b00101),
        BranchOnIndicatorsOpCode::BG => u5!(0b00110),
        BranchOnIndicatorsOpCode::BLE => u5!(0b00111),
        BranchOnIndicatorsOpCode::BOV => u5!(0b01000),
        BranchOnIndicatorsOpCode::BNOV => u5!(0b01001),
        BranchOnIndicatorsOpCode::BBT => u5!(0b01010),
        BranchOnIndicatorsOpCode::BBF => u5!(0b01011),
        BranchOnIndicatorsOpCode::BCT => u5!(0b01100),
        BranchOnIndicatorsOpCode::BCF => u5!(0b01101),
        BranchOnIndicatorsOpCode::BIOT => u5!(0b01110),
        BranchOnIndicatorsOpCode::BIOF => u5!(0b01111),
        BranchOnIndicatorsOpCode::BAL => u5!(0b10000),
        BranchOnIndicatorsOpCode::BAGE => u5!(0b10001),
        BranchOnIndicatorsOpCode::BE => u5!(0b10010),
        BranchOnIndicatorsOpCode::BNE => u5!(0b10011),
        BranchOnIndicatorsOpCode::BAG => u5!(0b10100),
        BranchOnIndicatorsOpCode::BALE => u5!(0b10101),
        BranchOnIndicatorsOpCode::BSU => u5!(0b10110),
        BranchOnIndicatorsOpCode::BSE => u5!(0b10111),
        BranchOnIndicatorsOpCode::NOP => u5!(0b11110),
        BranchOnIndicatorsOpCode::B => u5!(0b11111),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::statements::AddressExpression;

    #[test]
    fn codegen_branch_on_indicators_instructions_succ() {
        let tests = [
            (
                (
                    BranchOnIndicatorsOpCode::BL,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                    0,
                ),
                vec![0b0000001000000000, 0x1234],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BGE,
                    BranchLocation::LongDisplacement(AddressExpression::Immediate(0x1234)),
                    0,
                ),
                vec![0b0000001010000001, 0x1234],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BG,
                    BranchLocation::ShortDisplacement(AddressExpression::Immediate(0x1234)),
                    0x1230,
                ),
                vec![0b0000001100000100],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BLE,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                    1000,
                ),
                vec![0b0000001110000000, 0x1234],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BOV,
                    BranchLocation::LongDisplacement(AddressExpression::Immediate(500)),
                    1000,
                ),
                vec![0b0000010000000001, 0xFE0C],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BOV,
                    BranchLocation::ShortDisplacement(AddressExpression::Immediate(999)),
                    1000,
                ),
                vec![0b0000010001111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BNOV,
                    BranchLocation::ShortDisplacement(AddressExpression::Immediate(112)),
                    100,
                ),
                vec![0b0000010010001100],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BBT,
                    BranchLocation::LongDisplacement(AddressExpression::Immediate(100)),
                    112,
                ),
                vec![0b0000010100000001, 0xFFF4],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BBF,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000010111111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BCT,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000011001111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BCF,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000011011111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BIOT,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000011101111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BIOF,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000011111111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BIOF,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000011111111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BAL,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000100001111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BAGE,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000100011111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BE,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000100101111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BNE,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000100111111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BAG,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000101001111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BALE,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000101011111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BSU,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000101101111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BSE,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000101111111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::B,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000111111111111],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::NOP,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
                    1000,
                ),
                vec![0b0000111101111111],
            ),
        ];

        let label_table: HashMap<String, u64> = HashMap::new();

        for ((op, branchloc, cur_addr), exp) in tests {
            assert_eq!(
                codegen_branch_on_indicators(&op, &branchloc, cur_addr, &label_table).unwrap(),
                exp
            );
        }
    }
}
