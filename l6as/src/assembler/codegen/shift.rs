use crate::{assembler::DataRegister, logging::AssemblerErrorKind};

use bit_struct::*;

use crate::assembler::statements::{ShiftLongOpCode, ShiftShortOpCode};

use super::common::get_data_register_value;

bit_struct! {
    pub struct ShiftShortInstructionWord(u16) {
        header: u1,
        reg: u3,
        pad: u4,
        op: u4,
        dist: u4,
    }
    pub struct ShiftLongInstructionWord(u16) {
        header: u1,
        reg: u3,
        pad: u4,
        op: u3,
        dist: u5,
    }
}

/// Generaete code for a Shift Short instruction
pub fn codegen_shift_short(
    op: &ShiftShortOpCode,
    reg: &DataRegister,
    dist: u64,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Build instruction word
    let inst_word = ShiftShortInstructionWord::new(
        u1!(0),
        get_data_register_value(reg),
        u4!(0b0000),
        get_shift_short_op_value(op),
        get_shift_short_dist_field(dist)?,
    );

    Ok(vec![inst_word.raw()])
}

/// Generaete code for a Shift Long instruction
pub fn codegen_shift_long(
    op: &ShiftLongOpCode,
    reg: &DataRegister,
    dist: u64,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Build instruction word
    let inst_word = ShiftLongInstructionWord::new(
        u1!(0),
        get_data_register_value(reg),
        u4!(0b0000),
        get_shift_long_op_value(op),
        get_shift_long_dist_field(dist)?,
    );

    Ok(vec![inst_word.raw()])
}

fn get_shift_short_op_value(op: &ShiftShortOpCode) -> u4 {
    match op {
        ShiftShortOpCode::SOL => u4!(0b0000),
        ShiftShortOpCode::SCL => u4!(0b0001),
        ShiftShortOpCode::SAL => u4!(0b0100),
        ShiftShortOpCode::DCL => u4!(0b0011),
        ShiftShortOpCode::SOR => u4!(0b0100),
        ShiftShortOpCode::SCR => u4!(0b0101),
        ShiftShortOpCode::SAR => u4!(0b0110),
        ShiftShortOpCode::DCR => u4!(0b0111),
    }
}

fn get_shift_long_op_value(op: &ShiftLongOpCode) -> u3 {
    match op {
        ShiftLongOpCode::DOL => u3!(0b100),
        ShiftLongOpCode::DAL => u3!(0b101),
        ShiftLongOpCode::DOR => u3!(0b110),
        ShiftLongOpCode::DAR => u3!(0b111),
    }
}

fn get_shift_short_dist_field(dist: u64) -> Result<u4, AssemblerErrorKind> {
    // Distance into u8
    let distu8 = match TryInto::<u8>::try_into(dist) {
        Ok(val) => val,
        Err(_) => return Err(AssemblerErrorKind::ShiftDistanceOutOfRange(dist)),
    };

    // Distance into u4
    match u4::new(distu8) {
        Some(res) => Ok(res),
        None => Err(AssemblerErrorKind::ShiftDistanceOutOfRange(dist)),
    }
}

fn get_shift_long_dist_field(dist: u64) -> Result<u5, AssemblerErrorKind> {
    // Distance into u8
    let distu8 = match TryInto::<u8>::try_into(dist) {
        Ok(val) => val,
        Err(_) => return Err(AssemblerErrorKind::ShiftDistanceOutOfRange(dist)),
    };

    // Distance into u4
    match u5::new(distu8) {
        Some(res) => Ok(res),
        None => Err(AssemblerErrorKind::ShiftDistanceOutOfRange(dist)),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::assembler::statements::AddressExpression;

//     #[test]
//     fn codegen_branch_on_indicators_instructions_succ() {
//         let tests = [
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BL,
//                     BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
//                     0,
//                 ),
//                 vec![0b0000001000000000, 0x1234],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BGE,
//                     BranchLocation::LongDisplacement(AddressExpression::Immediate(0x1234)),
//                     0,
//                 ),
//                 vec![0b0000001010000001, 0x1234],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BG,
//                     BranchLocation::ShortDisplacement(AddressExpression::Immediate(0x1234)),
//                     0x1230,
//                 ),
//                 vec![0b0000001100000100],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BLE,
//                     BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
//                     1000,
//                 ),
//                 vec![0b0000001110000000, 0x1234],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BOV,
//                     BranchLocation::LongDisplacement(AddressExpression::Immediate(500)),
//                     1000,
//                 ),
//                 vec![0b0000010000000001, 0xFE0C],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BOV,
//                     BranchLocation::ShortDisplacement(AddressExpression::Immediate(999)),
//                     1000,
//                 ),
//                 vec![0b0000010001111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BNOV,
//                     BranchLocation::ShortDisplacement(AddressExpression::Immediate(112)),
//                     100,
//                 ),
//                 vec![0b0000010010001100],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BBT,
//                     BranchLocation::LongDisplacement(AddressExpression::Immediate(100)),
//                     112,
//                 ),
//                 vec![0b0000010100000001, 0xFFF4],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BBF,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000010111111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BCT,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000011001111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BCF,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000011011111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BIOT,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000011101111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BIOF,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000011111111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BIOF,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000011111111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BAL,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000100001111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BAGE,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000100011111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BE,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000100101111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BNE,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000100111111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BAG,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000101001111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BALE,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000101011111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BSU,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000101101111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::BSE,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000101111111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::B,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000111111111111],
//             ),
//             (
//                 (
//                     BranchOnIndicatorsOpCode::NOP,
//                     BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(-1)),
//                     1000,
//                 ),
//                 vec![0b0000111101111111],
//             ),
//         ];

//         let label_table: HashMap<String, u64> = HashMap::new();

//         for ((op, branchloc, cur_addr), exp) in tests {
//             assert_eq!(
//                 codegen_branch_on_indicators(&op, &branchloc, cur_addr, &label_table).unwrap(),
//                 exp
//             );
//         }
//     }
// }
