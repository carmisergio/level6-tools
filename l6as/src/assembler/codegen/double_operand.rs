use std::collections::HashMap;

use crate::{
    assembler::statements::{AddressSyllable, DoubleOperandOpCode, Register},
    logging::AssemblerErrorKind,
};

use bit_struct::*;

use super::{
    address_syllable::get_address_syllable_field_value,
    common::{get_generic_register_value, get_maskword_value},
};

bit_struct! {
    pub struct DoubleOperandInstructionWord(u16) {
        header: u1,
        reg: u3,
        op: u5,
        addr_syl: u7,
    }
}

/// Generaete code for a Double Operand  instruction
pub fn codegen_double_operand(
    op: &DoubleOperandOpCode,
    reg: &Register,
    addr_syl: &AddressSyllable,
    mask: &Option<i128>,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Process address syllable
    let (addr_syl_field, mut addr_syl_extra_words) =
        get_address_syllable_field_value(addr_syl, cur_addr, &label_table)?;

    // Build instruction word
    let inst_word = DoubleOperandInstructionWord::new(
        u1!(0b1),
        get_generic_register_value(reg),
        get_double_operand_op_value(op),
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

fn get_double_operand_op_value(op: &DoubleOperandOpCode) -> u5 {
    match op {
        DoubleOperandOpCode::LDR => u5!(0b10000),
        DoubleOperandOpCode::STR => u5!(0b11110),
        DoubleOperandOpCode::SRM => u5!(0b10101),
        DoubleOperandOpCode::SWR => u5!(0b11100),
        DoubleOperandOpCode::CMR => u5!(0b10010),
        DoubleOperandOpCode::ADD => u5!(0b10100),
        DoubleOperandOpCode::SUB => u5!(0b00100),
        DoubleOperandOpCode::MUL => u5!(0b10110),
        DoubleOperandOpCode::DIV => u5!(0b00110),
        DoubleOperandOpCode::OR => u5!(0b01000),
        DoubleOperandOpCode::XOR => u5!(0b01100),
        DoubleOperandOpCode::AND => u5!(0b01010),
        DoubleOperandOpCode::LDH => u5!(0b00001),
        DoubleOperandOpCode::STH => u5!(0b01111),
        DoubleOperandOpCode::CMH => u5!(0b00011),
        DoubleOperandOpCode::ORH => u5!(0b01001),
        DoubleOperandOpCode::XOH => u5!(0b01101),
        DoubleOperandOpCode::ANH => u5!(0b01011),
        DoubleOperandOpCode::LLH => u5!(0b00101),
        DoubleOperandOpCode::MTM => u5!(0b00000),
        DoubleOperandOpCode::STM => u5!(0b01110),
        DoubleOperandOpCode::LDB => u5!(0b11001),
        DoubleOperandOpCode::STB => u5!(0b11111),
        DoubleOperandOpCode::CMB => u5!(0b11011),
        DoubleOperandOpCode::SWB => u5!(0b11101),
        DoubleOperandOpCode::LAB => u5!(0b10111),
        DoubleOperandOpCode::LNJ => u5!(0b00111),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::statements::{
        AddressExpression, BaseRegister, DataRegister, ImmediateAddress, ImmediateAddressMode,
        ModeControlRegister,
    };

    #[test]
    fn codegen_branch_on_indicators_instructions_succ() {
        let tests = [
            (
                (
                    DoubleOperandOpCode::LDR,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001100001010010],
            ),
            (
                (
                    DoubleOperandOpCode::STR,
                    Register::Data(DataRegister::R3),
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x1234)),
                    )),
                    None,
                    100,
                ),
                vec![0b1011111100000000, 0x1234],
            ),
            (
                (
                    DoubleOperandOpCode::SRM,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    Some(0xAAAA),
                    100,
                ),
                vec![0b1001101011010010, 0xAAAA],
            ),
            (
                (
                    DoubleOperandOpCode::SWR,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001111001010010],
            ),
            (
                (
                    DoubleOperandOpCode::CMR,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001100101010010],
            ),
            (
                (
                    DoubleOperandOpCode::ADD,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001101001010010],
            ),
            (
                (
                    DoubleOperandOpCode::SUB,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001001001010010],
            ),
            (
                (
                    DoubleOperandOpCode::MUL,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001101101010010],
            ),
            (
                (
                    DoubleOperandOpCode::DIV,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001001101010010],
            ),
            (
                (
                    DoubleOperandOpCode::OR,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001010001010010],
            ),
            (
                (
                    DoubleOperandOpCode::XOR,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001011001010010],
            ),
            (
                (
                    DoubleOperandOpCode::AND,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001010101010010],
            ),
            (
                (
                    DoubleOperandOpCode::LDH,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001000011010010],
            ),
            (
                (
                    DoubleOperandOpCode::STH,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001011111010010],
            ),
            (
                (
                    DoubleOperandOpCode::CMH,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001000111010010],
            ),
            (
                (
                    DoubleOperandOpCode::ORH,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001010011010010],
            ),
            (
                (
                    DoubleOperandOpCode::XOH,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001011011010010],
            ),
            (
                (
                    DoubleOperandOpCode::ANH,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001010111010010],
            ),
            (
                (
                    DoubleOperandOpCode::LLH,
                    Register::Data(DataRegister::R1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001001011010010],
            ),
            (
                (
                    DoubleOperandOpCode::MTM,
                    Register::ModeControl(ModeControlRegister::M1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001000001010010],
            ),
            (
                (
                    DoubleOperandOpCode::STM,
                    Register::ModeControl(ModeControlRegister::M1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001011101010010],
            ),
            (
                (
                    DoubleOperandOpCode::LDB,
                    Register::Base(BaseRegister::B1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001110011010010],
            ),
            (
                (
                    DoubleOperandOpCode::STB,
                    Register::Base(BaseRegister::B1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001111111010010],
            ),
            (
                (
                    DoubleOperandOpCode::CMB,
                    Register::Base(BaseRegister::B1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001110111010010],
            ),
            (
                (
                    DoubleOperandOpCode::SWB,
                    Register::Base(BaseRegister::B1),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R2)),
                    None,
                    100,
                ),
                vec![0b1001111011010010],
            ),
            (
                (
                    DoubleOperandOpCode::LAB,
                    Register::Base(BaseRegister::B1),
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x0000)),
                    )),
                    None,
                    100,
                ),
                vec![0b1001101110000000, 0x0000],
            ),
            (
                (
                    DoubleOperandOpCode::LNJ,
                    Register::Base(BaseRegister::B1),
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x0000)),
                    )),
                    None,
                    100,
                ),
                vec![0b1001001110000000, 0x0000],
            ),
        ];

        let label_table: HashMap<String, u64> = HashMap::new();

        for ((op, reg, addr_syl, mask, cur_addr), exp) in tests {
            assert_eq!(
                codegen_double_operand(&op, &reg, &addr_syl, &mask, cur_addr, &label_table)
                    .unwrap(),
                exp
            );
        }
    }
}
