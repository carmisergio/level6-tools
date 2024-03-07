use crate::{
    assembler::statements::{DataRegister, ShortValueImmediateOpCode},
    logging::AssemblerErrorKind,
};

use bit_struct::*;

use super::common::get_data_register_value;

bit_struct! {
    pub struct ShortValueImmediateInstructionWord(u16) {
        header: u1,
        reg: u3,
        op: u4,
        val: u8,
    }
}

/// Generaete code for a Short Value Immediate  instruction
pub fn codegen_short_value_immediate(
    op: &ShortValueImmediateOpCode,
    reg: &DataRegister,
    val: i128,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Build instruction word
    let inst_word = ShortValueImmediateInstructionWord::new(
        u1!(0b0000),
        get_data_register_value(reg),
        get_short_value_immediate_op_value(op),
        get_short_immediate_value_field(val)?,
    );

    Ok(vec![inst_word.raw()])
}

fn get_short_value_immediate_op_value(op: &ShortValueImmediateOpCode) -> u4 {
    match op {
        ShortValueImmediateOpCode::LDV => u4!(0b1100),
        ShortValueImmediateOpCode::CMV => u4!(0b1101),
        ShortValueImmediateOpCode::ADV => u4!(0b1110),
        ShortValueImmediateOpCode::MLV => u4!(0b1111),
    }
}

fn get_short_immediate_value_field(val: i128) -> Result<u8, AssemblerErrorKind> {
    if val >= 0 {
        // Raw value into u8
        match TryInto::<u8>::try_into(val) {
            Ok(val) => Ok(val),
            Err(_) => Err(AssemblerErrorKind::ShortImmediateValueOutOfRange(val)),
        }
    } else {
        // Two's complemented i8 into u8
        match TryInto::<i8>::try_into(val) {
            Ok(val) => Ok(u8::from_be_bytes(val.to_be_bytes())),
            Err(_) => Err(AssemblerErrorKind::ShortImmediateValueOutOfRange(val)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codegen_branch_on_indicators_instructions_succ() {
        let tests = [
            (
                (ShortValueImmediateOpCode::LDV, DataRegister::R1, 0),
                vec![0b0001110000000000],
            ),
            (
                (ShortValueImmediateOpCode::CMV, DataRegister::R2, 0xFF),
                vec![0b0010110111111111],
            ),
            (
                (ShortValueImmediateOpCode::ADV, DataRegister::R3, -1),
                vec![0b0011111011111111],
            ),
            (
                (ShortValueImmediateOpCode::MLV, DataRegister::R4, 170),
                vec![0b0100111110101010],
            ),
        ];

        for ((op, branchloc, val), exp) in tests {
            assert_eq!(
                codegen_short_value_immediate(&op, &branchloc, val).unwrap(),
                exp
            );
        }
    }
}
