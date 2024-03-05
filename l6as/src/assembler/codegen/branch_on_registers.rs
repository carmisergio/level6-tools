use std::collections::HashMap;

use crate::{
    assembler::statements::{BranchOnRegistersOpCode, DataRegister},
    logging::AssemblerErrorKind,
};

use bit_struct::*;

use crate::assembler::statements::BranchLocation;

use super::common::get_branch_location_field_value;

bit_struct! {
    pub struct BranchOnRegistersInstructionWord(u16) {
        header: u1,
        reg: u3,
        op: u5,
        branchloc: i7
    }
}

/// Generaete code for a Branch on Indicators instruction
pub fn codegen_branch_on_registers(
    op: &BranchOnRegistersOpCode,
    reg: &DataRegister,
    branchloc: &BranchLocation,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Generate branch location field
    let (branchloc_field, mut extra_words) =
        get_branch_location_field_value(branchloc, cur_addr, label_table)?;

    // Build instruction word
    let inst_word = BranchOnRegistersInstructionWord::new(
        u1!(0b0000),
        get_data_register_value(reg),
        get_branch_on_registers_op_value(op),
        branchloc_field,
    );

    // Concatenate words
    let mut result = vec![inst_word.raw()];
    result.append(&mut extra_words); // Add potential extra word from branchloc field

    Ok(result)
}

fn get_branch_on_registers_op_value(op: &BranchOnRegistersOpCode) -> u5 {
    match op {
        BranchOnRegistersOpCode::BLZ => u5!(0b10000),
        BranchOnRegistersOpCode::BGEZ => u5!(0b10001),
        BranchOnRegistersOpCode::BEZ => u5!(0b10010),
        BranchOnRegistersOpCode::BNEZ => u5!(0b10011),
        BranchOnRegistersOpCode::BGZ => u5!(0b10100),
        BranchOnRegistersOpCode::BLEZ => u5!(0b10101),
        BranchOnRegistersOpCode::BEVN => u5!(0b10110),
        BranchOnRegistersOpCode::BODD => u5!(0b10111),
        BranchOnRegistersOpCode::BDEC => u5!(0b01110),
        BranchOnRegistersOpCode::BINC => u5!(0b01111),
    }
}

fn get_data_register_value(reg: &DataRegister) -> u3 {
    match reg {
        DataRegister::R1 => u3!(1),
        DataRegister::R2 => u3!(2),
        DataRegister::R3 => u3!(3),
        DataRegister::R4 => u3!(4),
        DataRegister::R5 => u3!(5),
        DataRegister::R6 => u3!(6),
        DataRegister::R7 => u3!(7),
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
                    BranchOnRegistersOpCode::BLZ,
                    DataRegister::R1,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                    0,
                ),
                vec![0b0001100000000000, 0x1234],
            ),
            (
                (
                    BranchOnRegistersOpCode::BGEZ,
                    DataRegister::R2,
                    BranchLocation::LongDisplacement(AddressExpression::Immediate(0x1234)),
                    0x100,
                ),
                vec![0b0010100010000001, 0x1134],
            ),
            (
                (
                    BranchOnRegistersOpCode::BEZ,
                    DataRegister::R3,
                    BranchLocation::ShortDisplacement(AddressExpression::Immediate(0x102)),
                    0x100,
                ),
                vec![0b0011100100000010],
            ),
            (
                (
                    BranchOnRegistersOpCode::BNEZ,
                    DataRegister::R4,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-10)),
                    0xFFF,
                ),
                vec![0b0100100111110110],
            ),
            (
                (
                    BranchOnRegistersOpCode::BGZ,
                    DataRegister::R4,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-10)),
                    0xFFF,
                ),
                vec![0b0100101001110110],
            ),
            (
                (
                    BranchOnRegistersOpCode::BLEZ,
                    DataRegister::R5,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-10)),
                    0xFFF,
                ),
                vec![0b0101101011110110],
            ),
            (
                (
                    BranchOnRegistersOpCode::BEVN,
                    DataRegister::R6,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-10)),
                    0xFFF,
                ),
                vec![0b0110101101110110],
            ),
            (
                (
                    BranchOnRegistersOpCode::BODD,
                    DataRegister::R7,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-10)),
                    0xFFF,
                ),
                vec![0b0111101111110110],
            ),
            (
                (
                    BranchOnRegistersOpCode::BDEC,
                    DataRegister::R7,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-10)),
                    0xFFF,
                ),
                vec![0b0111011101110110],
            ),
            (
                (
                    BranchOnRegistersOpCode::BINC,
                    DataRegister::R7,
                    BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-10)),
                    0xFFF,
                ),
                vec![0b0111011111110110],
            ),
        ];

        let label_table: HashMap<String, u64> = HashMap::new();

        for ((op, branchloc, reg, cur_addr), exp) in tests {
            assert_eq!(
                codegen_branch_on_registers(&op, &branchloc, &reg, cur_addr, &label_table).unwrap(),
                exp
            );
        }
    }

    #[test]
    fn get_data_register_value_succ() {
        let tests = [
            (DataRegister::R1, u3!(0b001)),
            (DataRegister::R2, u3!(0b010)),
            (DataRegister::R3, u3!(0b011)),
            (DataRegister::R4, u3!(0b100)),
            (DataRegister::R5, u3!(0b101)),
            (DataRegister::R6, u3!(0b110)),
            (DataRegister::R7, u3!(0b111)),
        ];

        for (input, exp) in tests {
            assert_eq!(get_data_register_value(&input), exp);
        }
    }
}
