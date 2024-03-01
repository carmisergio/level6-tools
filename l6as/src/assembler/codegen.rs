use std::{collections::HashMap, thread::current};

use proc_bitfield::bitfield;

use crate::logging::AssemblerErrorKind;

use super::statements::{AddressExpression, BranchLocation, BranchOnIndicatorsOpCode, Statement};

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct BranchOnIndicatorsInstructionWord(pub u16): Debug, FromRaw, IntoRaw, DerefRaw {
        pub header: u8 @ 0; 4,
        pub op: u8 @ 4; 5,
        pub branchloc: i8 @ 9; 7,
    }
}

/// Generate raw words for one statement
pub fn codegen(
    statement: &Statement,
    cur_addr: u128,
    label_table: &HashMap<String, u128>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Compute different size depending on the kind of statement
    match statement {
        Statement::Org(_) => Ok(vec![]),
        Statement::BranchOnIndicators(op, branchloc) => {
            codegen_branch_on_instructions(op, branchloc, cur_addr, label_table)
        }
    }
}

/// Generaete code for a Branch on Indicators instruction
fn codegen_branch_on_instructions(
    op: &BranchOnIndicatorsOpCode,
    branchloc: &BranchLocation,
    cur_addr: u128,
    label_table: &HashMap<String, u128>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Instruction format
    let mut inst_word = BranchOnIndicatorsInstructionWord(0);
    inst_word.set_header(0b0000); // Instruction header

    // Generate op code
    inst_word.set_op(get_branch_on_instructions_op_value(op));

    // Generate branch location field
    let (branchloc_field, mut extra_words) =
        get_branch_location_field_value(branchloc, cur_addr, label_table)?;

    // Concatenate words
    let mut result = vec![inst_word.to_be()];
    result.append(&mut extra_words); // Add potential extra word from branchloc field

    panic!();

    Ok(result)
}

fn get_branch_on_instructions_op_value(op: &BranchOnIndicatorsOpCode) -> u8 {
    match op {
        BranchOnIndicatorsOpCode::BL => 0b00100,
        BranchOnIndicatorsOpCode::BGE => 0b00101,
        BranchOnIndicatorsOpCode::BG => 0b00110,
        BranchOnIndicatorsOpCode::BLE => 0b00111,
        BranchOnIndicatorsOpCode::BOV => 0b01000,
        BranchOnIndicatorsOpCode::BNOV => 0b01001,
        BranchOnIndicatorsOpCode::BBT => 0b01010,
        BranchOnIndicatorsOpCode::BBF => 0b01011,
        BranchOnIndicatorsOpCode::BCT => 0b01100,
        BranchOnIndicatorsOpCode::BCF => 0b01101,
        BranchOnIndicatorsOpCode::BIOT => 0b01110,
        BranchOnIndicatorsOpCode::BIOF => 0b01110,
        BranchOnIndicatorsOpCode::BAL => 0b10000,
        BranchOnIndicatorsOpCode::BAGE => 0b10001,
        BranchOnIndicatorsOpCode::BE => 0b10010,
        BranchOnIndicatorsOpCode::BNE => 0b10011,
        BranchOnIndicatorsOpCode::BAG => 0b10100,
        BranchOnIndicatorsOpCode::BALE => 0b10101,
        BranchOnIndicatorsOpCode::BSU => 0b10110,
        BranchOnIndicatorsOpCode::BSE => 0b10111,
        BranchOnIndicatorsOpCode::NOP => 0b11110,
        BranchOnIndicatorsOpCode::B => 0b11111,
    }
}

fn get_branch_location_field_value(
    branchloc: &BranchLocation,
    cur_addr: u128,
    label_table: &HashMap<String, u128>,
) -> Result<(i8, Vec<u16>), AssemblerErrorKind> {
    let (value, extra_words) = match branchloc {
        // Absolute barnch location
        BranchLocation::Absolute(addr_exp) => {
            // Get absolute address from address expression
            let addr = resolve_address_expression(addr_exp, cur_addr, label_table)?;

            // If address is too big
            if addr > u16::MAX as u128 {
                return Err(AssemblerErrorKind::AddressOutOfRange(addr));
            }

            (0, vec![addr as u16])
        }
        // Long displacement branch location
        BranchLocation::LongDisplacement(addr_exp) => {
            // Get absolute address from address expression
            let addr = resolve_address_expression(addr_exp, cur_addr, label_table)?;

            // Calculate displacement
            let displacement = addr as i128 - cur_addr as i128;

            // Check displacement distance
            if displacement > 32767 || displacement < -32768 {
                return Err(AssemblerErrorKind::LongDisplacementOutOfRange(displacement));
            }

            (1, vec![displacement as u16])
        }
        // Long displacement branch location
        BranchLocation::ShortDisplacement(addr_exp) => {
            // Get absolute address from address expression
            let addr = resolve_address_expression(addr_exp, cur_addr, label_table)?;

            // Calculate displacement
            let displacement = addr as i128 - cur_addr as i128;

            // Check displacement distance
            if displacement > 63 || displacement < -64 {
                return Err(AssemblerErrorKind::ShortDisplacementOutOfRange(
                    displacement,
                ));
            }
            if displacement == 0 || displacement == 1 {
                return Err(AssemblerErrorKind::ShortDisplacementMustNotBe0Or1);
            }

            (displacement as i8, vec![])
        }
    };

    Ok((value, extra_words))
}

fn resolve_address_expression(
    addr_exp: &AddressExpression,
    cur_addr: u128,
    label_table: &HashMap<String, u128>,
) -> Result<u128, AssemblerErrorKind> {
    // TODO do address expression resolution
    Ok(0)
}

#[cfg(test)]
mod tests {
    use crate::assembler::statements::AddressExpression;

    use super::*;

    // #[test]
    // fn codegen_branch_on_instructions_succ() {
    //     let codegen_res = codegen_branch_on_instructions(
    //         &BranchOnIndicatorsOpCode::B,
    //         &BranchLocation::IndirectDisplacement(AddressExpression::Immediate(0x00)),
    //         0,
    //         ,
    //     );
    // }
}
