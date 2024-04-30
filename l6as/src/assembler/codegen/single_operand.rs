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
    use crate::assembler::statements::{
        AddressExpression, BRelativeAddress, BRelativeAddressMode, BaseRegister, DataRegister,
        ImmediateAddress, ImmediateAddressMode, IncDec, PRelativeAddress, Register,
    };

    use super::*;

    #[test]
    fn codegen_branch_on_indicators_instructions_succ() {
        let tests = [
            (
                (
                    SingleOperandOpCode::INC,
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R1)),
                    None,
                    100,
                ),
                vec![0b1000101011010001],
            ),
            (
                (
                    SingleOperandOpCode::CMN,
                    AddressSyllable::RegisterAddressing(Register::Base(BaseRegister::B2)),
                    None,
                    0,
                ),
                vec![0b1000110111010010],
            ),
            (
                (
                    SingleOperandOpCode::DEC,
                    AddressSyllable::ImmediateOperand(0x1234),
                    None,
                    0,
                ),
                vec![0b1000100011110000, 0x1234],
            ),
            (
                (
                    SingleOperandOpCode::NEG,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x4567)),
                    )),
                    None,
                    0,
                ),
                vec![0b1000001000000000, 0x4567],
            ),
            (
                (
                    SingleOperandOpCode::CPL,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Indirect(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x2939)),
                    )),
                    None,
                    100,
                ),
                vec![0b1000011000001000, 0x2939],
            ),
            (
                (
                    SingleOperandOpCode::CL,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Indexed(
                            AddressExpression::Immediate(0x2000),
                            DataRegister::R1,
                        ),
                    )),
                    None,
                    100,
                ),
                vec![0b1000011100010000, 0x2000],
            ),
            (
                (
                    SingleOperandOpCode::CLH,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Indirect(
                        ImmediateAddress::Indexed(
                            AddressExpression::Immediate(0x2000),
                            DataRegister::R2,
                        ),
                    )),
                    None,
                    0,
                ),
                vec![0b1000011110101000, 0x2000],
            ),
            (
                (
                    SingleOperandOpCode::CMZ,
                    AddressSyllable::BRelative(BRelativeAddressMode::Direct(
                        BRelativeAddress::Simple(BaseRegister::B1),
                    )),
                    None,
                    0,
                ),
                vec![0b1000100110000001],
            ),
            (
                (
                    SingleOperandOpCode::CAD,
                    AddressSyllable::BRelative(BRelativeAddressMode::Indirect(
                        BRelativeAddress::Simple(BaseRegister::B2),
                    )),
                    None,
                    0,
                ),
                vec![0b1000111010001010],
            ),
            (
                (
                    SingleOperandOpCode::STS,
                    AddressSyllable::BRelative(BRelativeAddressMode::Direct(
                        BRelativeAddress::Displacement(BaseRegister::B3, 100),
                    )),
                    None,
                    200,
                ),
                vec![0b1000110001000011, 100],
            ),
            (
                (
                    SingleOperandOpCode::JMP,
                    AddressSyllable::BRelative(BRelativeAddressMode::Indirect(
                        BRelativeAddress::Displacement(BaseRegister::B4, -50),
                    )),
                    None,
                    200,
                ),
                vec![
                    0b1000001111001100,
                    u16::from_be_bytes((-50 as i16).to_be_bytes()),
                ],
            ),
            (
                (
                    SingleOperandOpCode::ENT,
                    AddressSyllable::BRelative(BRelativeAddressMode::Direct(
                        BRelativeAddress::Indexed(BaseRegister::B5, DataRegister::R3),
                    )),
                    None,
                    200,
                ),
                vec![0b1000101110110101],
            ),
            (
                (
                    SingleOperandOpCode::LEV,
                    AddressSyllable::BRelative(BRelativeAddressMode::Indirect(
                        BRelativeAddress::Indexed(BaseRegister::B6, DataRegister::R1),
                    )),
                    Some(0xAAAA),
                    200,
                ),
                vec![0b1000111000011110, 0xAAAA],
            ),
            (
                (
                    SingleOperandOpCode::SAVE,
                    AddressSyllable::BRelative(BRelativeAddressMode::PushPop(
                        BaseRegister::B7,
                        IncDec::Increment,
                    )),
                    Some(0xBBBB),
                    300,
                ),
                vec![0b1000111101110111, 0xBBBB],
            ),
            (
                (
                    SingleOperandOpCode::RSTR,
                    AddressSyllable::BRelative(BRelativeAddressMode::PushPop(
                        BaseRegister::B1,
                        IncDec::Decrement,
                    )),
                    Some(0xCCCC),
                    300,
                ),
                vec![0b1000111111100001, 0xCCCC],
            ),
            (
                (
                    SingleOperandOpCode::LB,
                    AddressSyllable::BRelative(BRelativeAddressMode::IncDecIndexed(
                        BaseRegister::B2,
                        DataRegister::R2,
                        IncDec::Increment,
                    )),
                    Some(0x1234),
                    300,
                ),
                vec![0b1000001011101110, 0x1234],
            ),
            (
                (
                    SingleOperandOpCode::LBF,
                    AddressSyllable::BRelative(BRelativeAddressMode::IncDecIndexed(
                        BaseRegister::B3,
                        DataRegister::R3,
                        IncDec::Decrement,
                    )),
                    Some(0x1234),
                    300,
                ),
                vec![0b1000100001111011, 0x1234],
            ),
            (
                (
                    SingleOperandOpCode::LBT,
                    AddressSyllable::PRelative(PRelativeAddress::Direct(
                        AddressExpression::Immediate(0x1234),
                    )),
                    Some(0x5555),
                    0x1000,
                ),
                vec![0b1000100101000000, 0x0234, 0x5555],
            ),
            (
                (
                    SingleOperandOpCode::LBC,
                    AddressSyllable::PRelative(PRelativeAddress::Indirect(
                        AddressExpression::Immediate(200),
                    )),
                    Some(0x5555),
                    1000,
                ),
                vec![
                    0b1000101101001000,
                    u16::from_be_bytes((-800 as i16).to_be_bytes()),
                    0x5555,
                ],
            ),
            (
                (
                    SingleOperandOpCode::LBS,
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R1)),
                    Some(0x0000),
                    0,
                ),
                vec![0b1000101001010001, 0x0000],
            ),
            (
                (
                    SingleOperandOpCode::AID,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x1000)),
                    )),
                    None,
                    0,
                ),
                vec![0b1000010000000000, 0x1000],
            ),
            (
                (
                    SingleOperandOpCode::LDI,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x1000)),
                    )),
                    None,
                    0,
                ),
                vec![0b1000110010000000, 0x1000],
            ),
            (
                (
                    SingleOperandOpCode::SDI,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x1000)),
                    )),
                    None,
                    0,
                ),
                vec![0b1000110100000000, 0x1000],
            ),
            (
                (
                    SingleOperandOpCode::SID,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x1000)),
                    )),
                    None,
                    0,
                ),
                vec![0b1000010010000000, 0x1000],
            ),
        ];

        let label_table: HashMap<String, u64> = HashMap::new();

        for ((op, addr_syl, mask, cur_addr), exp) in tests {
            assert_eq!(
                codegen_single_operand(&op, &addr_syl, &mask, cur_addr, &label_table).unwrap(),
                exp
            );
        }
    }
}
