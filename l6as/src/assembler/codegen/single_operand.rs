use std::collections::HashMap;

use crate::{
    assembler::statements::{AddressSyllable, SingleOperandOpCode},
    logging::AssemblerErrorKind,
};

use bit_struct::*;

use super::{address_syllable::get_address_syllable_field_value, common::get_maskword_value};

bit_struct! {
    pub struct SingleOperandInstructionWord(u16) {
        header: u4,
        op: u5,
        addr_syl: u7,
    }
}

/// Generaete code for a Single Operand  instruction
pub fn codegen_single_operand(
    op: &SingleOperandOpCode,
    addr_syl: &AddressSyllable,
    mask: &Option<i128>,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Process address syllable
    let (addr_syl_field, mut addr_syl_extra_words) =
        get_address_syllable_field_value(addr_syl, cur_addr, &label_table)?;

    // Build instruction word
    let inst_word = SingleOperandInstructionWord::new(
        u4!(0b1000),
        get_single_operand_op_value(op),
        addr_syl_field,
    );

    // Concatenate words
    let mut words = vec![inst_word.raw()];
    words.append(&mut addr_syl_extra_words);

    // Add mask if present
    if let Some(mask) = mask {
        words.push(get_maskword_value(*mask)?);
    }

    Ok(words)
}

fn get_single_operand_op_value(op: &SingleOperandOpCode) -> u5 {
    match op {
        SingleOperandOpCode::INC => u5!(0b10101),
        SingleOperandOpCode::DEC => u5!(0b10001),
        SingleOperandOpCode::NEG => u5!(0b00100),
        SingleOperandOpCode::CPL => u5!(0b01100),
        SingleOperandOpCode::CL => u5!(0b01110),
        SingleOperandOpCode::CLH => u5!(0b01111),
        SingleOperandOpCode::CMZ => u5!(0b10011),
        SingleOperandOpCode::CMN => u5!(0b11011),
        SingleOperandOpCode::CAD => u5!(0b11101),
        SingleOperandOpCode::STS => u5!(0b11000),
        SingleOperandOpCode::JMP => u5!(0b00111),
        SingleOperandOpCode::ENT => u5!(0b10111),
        SingleOperandOpCode::LEV => u5!(0b11100),
        SingleOperandOpCode::SAVE => u5!(0b11110),
        SingleOperandOpCode::RSTR => u5!(0b11111),
        SingleOperandOpCode::LB => u5!(0b00101),
        SingleOperandOpCode::LBF => u5!(0b10000),
        SingleOperandOpCode::LBT => u5!(0b10010),
        SingleOperandOpCode::LBC => u5!(0b10110),
        SingleOperandOpCode::LBS => u5!(0b10100),
        SingleOperandOpCode::AID => u5!(0b01000),
        SingleOperandOpCode::LDI => u5!(0b11001),
        SingleOperandOpCode::SDI => u5!(0b11010),
        SingleOperandOpCode::SID => u5!(0b01001),
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use crate::assembler::statements::{BRelativeAddress, BRelativeAddressMode, BaseRegister};

    use super::*;

    #[test]
    fn codegen_branch_on_indicators_instructions_succ() {
        let tests = [(
            (
                SingleOperandOpCode::INC,
                AddressSyllable::BRelative(BRelativeAddressMode::Direct(BRelativeAddress::Simple(
                    BaseRegister::B1,
                ))),
                None,
                100,
            ),
            vec![0b1000101010000001],
        )];

        let label_table: HashMap<String, u64> = HashMap::new();

        for ((op, addr_syl, mask, cur_addr), exp) in tests {
            assert_eq!(
                codegen_single_operand(&op, &addr_syl, &mask, cur_addr, &label_table).unwrap(),
                exp
            );
        }
    }
}
