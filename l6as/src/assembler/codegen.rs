use std::collections::HashMap;

use crate::logging::AssemblerErrorKind;

use bit_struct::*;

use super::statements::{AddressExpression, BranchLocation, BranchOnIndicatorsOpCode, Statement};

bit_struct! {
    pub struct BranchOnIndicatorsInstructionWord(u16) {
        header: u4,
        op: u5,
        branchloc: i7
    }
}

/// Generate raw words for one statement
pub fn codegen(
    statement: &Statement,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
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
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Generate branch location field
    let (branchloc_field, mut extra_words) =
        get_branch_location_field_value(branchloc, cur_addr, label_table)?;

    // Build instruction word
    let mut inst_word = BranchOnIndicatorsInstructionWord::new(
        u4!(0b0000),
        get_branch_on_instructions_op_value(op),
        branchloc_field,
    );

    // Concatenate words
    let mut result = vec![inst_word.raw()];
    result.append(&mut extra_words); // Add potential extra word from branchloc field

    Ok(result)
}
// fn codegen_branch_on_instructions(
//     op: &BranchOnIndicatorsOpCode,
//     branchloc: &BranchLocation,
//     cur_addr: u64,
//     label_table: &HashMap<String, u64>,
// ) -> Result<Vec<u16>, AssemblerErrorKind> {
//     // Instruction format
//     let mut inst_word = BranchOnIndicatorsInstructionWord(0);
//     inst_word.set_header(0b0000); // Instruction header

//     // Generate op code
//     inst_word.set_op(get_branch_on_instructions_op_value(op));

//     // Generate branch location field
//     let (branchloc_field, mut extra_words) =
//         get_branch_location_field_value(branchloc, cur_addr, label_table)?;
//     inst_word.set_branchloc(branchloc_field);

//     // Package instruction into word
//     let inst_word: u16 = inst_word.try_into().unwrap();

//     println!("{:#b}", inst_word);

//     // Concatenate words
//     let mut result = vec![inst_word.to_be()];
//     result.append(&mut extra_words); // Add potential extra word from branchloc field

//     Ok(result)
// }

fn get_branch_on_instructions_op_value(op: &BranchOnIndicatorsOpCode) -> u5 {
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
        BranchOnIndicatorsOpCode::BIOF => u5!(0b01110),
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

fn get_branch_location_field_value(
    branchloc: &BranchLocation,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<(i7, Vec<u16>), AssemblerErrorKind> {
    let (value, extra_words) = match branchloc {
        // Absolute barnch location
        BranchLocation::Absolute(addr_exp) => {
            // Get absolute address from address expression
            let addr = resolve_address_expression(addr_exp, cur_addr, label_table)?;

            // Verify address fits in field
            match TryInto::<u16>::try_into(addr) {
                Ok(addr) => (i7!(0), vec![addr]),
                Err(_) => return Err(AssemblerErrorKind::AddressOutOfRange(addr)),
            }
        }
        // Long displacement branch location
        BranchLocation::LongDisplacement(addr_exp) => {
            // Get absolute address from address expression
            let addr = resolve_address_expression(addr_exp, cur_addr, label_table)?;

            // Calculate displacement
            let displacement = addr as i128 - cur_addr as i128;

            // Fit displacement in 16 bits
            let displacement: i16 = match displacement.try_into() {
                Ok(res) => res,
                Err(_) => return Err(AssemblerErrorKind::LongDisplacementOutOfRange(displacement)),
            };

            (i7!(1), vec![twos_complement_u16(displacement)])
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

            // Fit displacement in 8 bits TODO find way to make this more elegant
            let displacement_i8: i8 = match displacement.try_into() {
                Ok(res) => res,
                Err(_) => return Err(AssemblerErrorKind::LongDisplacementOutOfRange(displacement)),
            };

            // Fit displacement in 7 bits
            let displacement_i7 = match i7::new(displacement_i8) {
                Some(res) => res,
                None => return Err(AssemblerErrorKind::LongDisplacementOutOfRange(displacement)),
            };

            (displacement_i7, vec![])
        }
    };

    Ok((value, extra_words))
}

/// Resolve address expression to an absolute address in the u64 address space
fn resolve_address_expression(
    addr_exp: &AddressExpression,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<u64, AssemblerErrorKind> {
    match addr_exp {
        // Immediate address expression
        AddressExpression::Immediate(addr) => Ok(*addr),

        // Word displacement addressing expression
        AddressExpression::WordDisplacement(disp) => match (cur_addr as i128 + disp).try_into() {
            Ok(addr) => Ok(addr),
            Err(_) => Err(AssemblerErrorKind::WordDisplacementOutOfRange(*disp)),
        },

        // Label address expression
        AddressExpression::Label(label) => {
            // Find label
            match label_table.get(label) {
                Some(addr) => Ok(*addr),
                None => Err(AssemblerErrorKind::UndefinedLabel(label.to_owned())),
            }
        }
    }
}

/// Returns u16 containing a two's complement encoded relative number
fn twos_complement_u16(input: i16) -> u16 {
    let bytes = input.to_be_bytes();
    u16::from_be_bytes(bytes)
}

/// Returns u8 containing a two's complement encoded relative number
fn twos_complement_u8(input: i8) -> u8 {
    let bytes = input.to_be_bytes();
    u8::from_be_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::statements::AddressExpression;

    #[test]
    fn resolve_address_expression_succ() {
        let tests = [
            (AddressExpression::Immediate(1234), 0, 1234),
            (AddressExpression::Immediate(u64::MAX), 1000, u64::MAX),
            (AddressExpression::WordDisplacement(10), 100, 110),
            (AddressExpression::WordDisplacement(-30), 5000, 4970),
            (
                AddressExpression::WordDisplacement(1),
                u64::MAX - 1,
                u64::MAX,
            ),
            (AddressExpression::WordDisplacement(-1), 1, 0),
            (AddressExpression::Label("LABEL1".to_owned()), 123, 1000),
            (
                AddressExpression::Label("albatross".to_owned()),
                0,
                0x9999999999999999,
            ),
        ];

        let label_table = HashMap::from([
            ("LABEL1".to_owned(), 1000),
            ("albatross".to_owned(), 0x9999999999999999),
        ]);

        for (input, cur_addr, expected) in tests {
            assert_eq!(
                resolve_address_expression(&input, cur_addr, &label_table).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn resolve_address_expression_err() {
        let tests = [
            (AddressExpression::WordDisplacement(-1), 0),
            (AddressExpression::WordDisplacement(1), u64::MAX),
            (AddressExpression::Label("notexist".to_owned()), 0),
        ];

        let label_table = HashMap::from([
            ("LABEL1".to_owned(), 1000),
            ("albatross".to_owned(), 0x9999999999999999),
        ]);

        for (input, cur_addr) in tests {
            resolve_address_expression(&input, cur_addr, &label_table).unwrap_err();
        }
    }

    #[test]
    fn twos_complement_u16_succ() {
        let tests = [
            (0, 0b0000000000000000),
            (12345, 0b0011000000111001),
            (-12345, 0b1100111111000111),
            (i16::MAX, 0b0111111111111111),
            (i16::MIN, 0b1000000000000000),
        ];

        for (input, exp) in tests {
            assert_eq!(twos_complement_u16(input), exp);
        }
    }

    #[test]
    fn twos_complement_u8_succ() {
        let tests = [
            (0, 0b00000000),
            (12, 0b000001100),
            (-12, 0b11110100),
            (i8::MAX, 0b01111111),
            (i8::MIN, 0b10000000),
        ];

        for (input, exp) in tests {
            assert_eq!(twos_complement_u8(input), exp);
        }
    }

    #[test]
    fn codegen_branch_on_indicators_instructions_succ() {
        let tests = [
            (
                (
                    BranchOnIndicatorsOpCode::BL,
                    BranchLocation::Absolute(AddressExpression::Immediate(1234)),
                    0,
                ),
                vec![0b0000001000000000, 1234],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BL,
                    BranchLocation::ShortDisplacement(AddressExpression::Immediate(112)),
                    100,
                ),
                vec![0b0000001000001100],
            ),
            (
                (
                    BranchOnIndicatorsOpCode::BL,
                    BranchLocation::ShortDisplacement(AddressExpression::Immediate(100)),
                    112,
                ),
                vec![0b0000001001110100],
            ),
        ];

        let label_table: HashMap<String, u64> = HashMap::new();

        for ((op, branchloc, cur_addr), exp) in tests {
            assert_eq!(
                codegen_branch_on_instructions(&op, &branchloc, cur_addr, &label_table).unwrap(),
                exp
            );
        }
    }

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
