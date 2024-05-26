use std::collections::HashMap;

use crate::{
    assembler::statements::{AddressSyllable, ChannelExpression, InputOutputOpCode},
    logging::AssemblerErrorKind,
};

use bit_struct::*;
use clap::error::Result;

use super::address_syllable::get_address_syllable_field_value;

bit_struct! {
    pub struct InputOutputInstructionWord(u16) {
        header: u4,
        op: u5,
        addr_syl: u7,
    }
    pub struct ChannelExpressionWordImmediate(u16) {
        chan: u10,
        func: u6,
    }
    pub struct ChannelExpressionWordAddrSyl(u16) {
        header: u9,
        addr_syl: u7,
    }
    pub struct InputOutputRangeWord(u16) {
        header: u9,
        func: u7,
    }
}

/// Generaete code for an Input Output instruction
pub fn codegen_input_output(
    op: &InputOutputOpCode,
    data_addr_syl: &AddressSyllable,
    chan_expr: &ChannelExpression,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Process data address syllable
    let (data_addr_syl_field, mut data_addr_syl_extra_words) =
        get_address_syllable_field_value(data_addr_syl, cur_addr, &label_table)?;

    // Build instruction word
    let inst_word = InputOutputInstructionWord::new(
        u4!(0b1000),
        get_input_output_op_value(op),
        data_addr_syl_field,
    );

    // Build channel expression word
    let mut chan_expr_words = get_channel_expression_words(chan_expr, cur_addr, label_table)?;

    // Concatenate words
    let mut words = vec![inst_word.raw()];
    words.append(&mut data_addr_syl_extra_words);
    words.append(&mut chan_expr_words);

    Ok(words)
}

/// Generaete code for an Input Output Load instruction
pub fn codegen_input_output_load(
    buffer_addr_syl: &AddressSyllable,
    chan_expr: &ChannelExpression,
    range_addr_syl: &AddressSyllable,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Process buffer address syllable
    let (buffer_addr_syl_field, mut buffer_addr_syl_extra_words) =
        get_address_syllable_field_value(buffer_addr_syl, cur_addr, &label_table)?;

    // Process range address syllable
    let (range_addr_syl_field, mut range_addr_syl_extra_words) =
        get_address_syllable_field_value(range_addr_syl, cur_addr, &label_table)?;

    // Build instruction word
    let inst_word =
        InputOutputInstructionWord::new(u4!(0b1000), u5!(0b00011), buffer_addr_syl_field);

    // Build channel expression word
    let mut chan_expr_words = get_channel_expression_words(chan_expr, cur_addr, label_table)?;

    // Build range word
    let range_word = InputOutputRangeWord::new(u9!(0b000000000), range_addr_syl_field);

    // Concatenate words
    let mut words = vec![inst_word.raw()];
    words.append(&mut buffer_addr_syl_extra_words);
    words.append(&mut chan_expr_words);
    words.push(range_word.raw());
    words.append(&mut range_addr_syl_extra_words);

    Ok(words)
}

fn get_input_output_op_value(op: &InputOutputOpCode) -> u5 {
    match op {
        InputOutputOpCode::IO => u5!(0b00000),
        InputOutputOpCode::IOH => u5!(0b00010),
    }
}

fn get_channel_expression_words(
    chan_expr: &ChannelExpression,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    match chan_expr {
        ChannelExpression::Immediate(chan, func) => {
            let ce_word = ChannelExpressionWordImmediate::new(
                get_chan_field_value(*chan)?,
                get_func_field_value(*func)?,
            );
            Ok(vec![ce_word.raw()])
        }
        ChannelExpression::AddressSyllable(addr_syl) => {
            // Process address syllable
            let (addr_syl_field, mut addr_syl_extra_words) =
                get_address_syllable_field_value(addr_syl, cur_addr, &label_table)?;

            // Consruct ChannelExpression word
            let ce_word = ChannelExpressionWordAddrSyl::new(u9!(0b000000000), addr_syl_field);
            // Concatenate words
            let mut words = vec![ce_word.raw()];
            words.append(&mut addr_syl_extra_words);

            Ok(words)
        }
    }
}

fn get_chan_field_value(chan: u64) -> Result<u10, AssemblerErrorKind> {
    // Channel into u16
    let chan16 = match TryInto::<u16>::try_into(chan) {
        Ok(val) => val,
        Err(_) => return Err(AssemblerErrorKind::ChannelOutOfRange(chan)),
    };

    // Channel into u10
    match u10::new(chan16) {
        Some(res) => Ok(res),
        None => Err(AssemblerErrorKind::ChannelOutOfRange(chan)),
    }
}

fn get_func_field_value(func: u64) -> Result<u6, AssemblerErrorKind> {
    // Func into u8
    let func8 = match TryInto::<u8>::try_into(func) {
        Ok(val) => val,
        Err(_) => return Err(AssemblerErrorKind::FunctionCodeOutOfRange(func)),
    };

    // Func into u6
    match u6::new(func8) {
        Some(res) => Ok(res),
        None => Err(AssemblerErrorKind::FunctionCodeOutOfRange(func)),
    }
}

#[cfg(test)]
mod tests {
    use crate::assembler::{
        statements::{
            AddressExpression, BRelativeAddress, BRelativeAddressMode, DataRegister,
            ImmediateAddress, ImmediateAddressMode, Register,
        },
        BaseRegister,
    };

    use super::*;

    #[test]
    fn codegen_input_output_instructions_succ() {
        let tests = [
            (
                (
                    InputOutputOpCode::IO,
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R1)),
                    ChannelExpression::Immediate(0x20, 0x18),
                    100,
                ),
                vec![0b1000000001010001, 0b0000100000011000],
            ),
            (
                (
                    InputOutputOpCode::IOH,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x1234)),
                    )),
                    ChannelExpression::AddressSyllable(AddressSyllable::ImmediateOperand(0xAAAA)),
                    100,
                ),
                vec![0b1000000100000000, 0x1234, 0b0000000001110000, 0xAAAA],
            ),
            (
                (
                    InputOutputOpCode::IO,
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0x1234)),
                    )),
                    ChannelExpression::AddressSyllable(AddressSyllable::RegisterAddressing(
                        Register::Data(DataRegister::R2),
                    )),
                    100,
                ),
                vec![0b1000000000000000, 0x1234, 0b0000000001010010],
            ),
        ];

        let label_table: HashMap<String, u64> = HashMap::new();

        for ((op, addr_syl, chan_expr, cur_addr), exp) in tests {
            assert_eq!(
                codegen_input_output(&op, &addr_syl, &chan_expr, cur_addr, &label_table).unwrap(),
                exp
            );
        }
    }

    #[test]
    fn codegen_input_output_load_instructions_succ() {
        let tests = [
            (
                (
                    AddressSyllable::BRelative(BRelativeAddressMode::Direct(
                        BRelativeAddress::Simple(BaseRegister::B1),
                    )),
                    ChannelExpression::Immediate(0x42, 0x09),
                    AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R3)),
                    100,
                ),
                vec![0b1000000110000001, 0b0001000010001001, 0b0000000001010011],
            ),
            (
                (
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0xBBBB)),
                    )),
                    ChannelExpression::AddressSyllable(AddressSyllable::ImmediateAddressing(
                        ImmediateAddressMode::Direct(ImmediateAddress::Simple(
                            AddressExpression::Immediate(0xCCCC),
                        )),
                    )),
                    AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                        ImmediateAddress::Simple(AddressExpression::Immediate(0xDDDD)),
                    )),
                    100,
                ),
                vec![
                    0b1000000110000000,
                    0xBBBB,
                    0b0000000000000000,
                    0xCCCC,
                    0b0000000000000000,
                    0xDDDD,
                ],
            ),
        ];

        let label_table: HashMap<String, u64> = HashMap::new();

        for ((buffer_addr_syl, chan_expr, range_addr_syl, cur_addr), exp) in tests {
            assert_eq!(
                codegen_input_output_load(
                    &buffer_addr_syl,
                    &chan_expr,
                    &range_addr_syl,
                    cur_addr,
                    &label_table
                )
                .unwrap(),
                exp
            );
        }
    }
}
