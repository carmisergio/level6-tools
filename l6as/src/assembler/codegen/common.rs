use std::collections::HashMap;

use crate::{assembler::statements::ModeControlRegister, logging::AssemblerErrorKind};

use bit_struct::*;

use crate::assembler::statements::{
    AddressExpression, BaseRegister, BranchLocation, DataRegister, Register, Statement,
};

use super::{
    branch_on_indicators::codegen_branch_on_indicators,
    branch_on_registers::codegen_branch_on_registers,
    data_definition::codegen_data_definition,
    double_operand::codegen_double_operand,
    generic::codegen_generic,
    input_output::{codegen_input_output, codegen_input_output_load},
    shift::{codegen_shift_long, codegen_shift_short},
    short_value_immediate::codegen_short_value_immediate,
    single_operand::codegen_single_operand,
};

/// Generate raw words for one statement
pub fn codegen(
    statement: &Statement,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Compute different size depending on the kind of statement
    match statement {
        Statement::Org(_) => Ok(vec![]),
        Statement::DataDefinition(size, values) => codegen_data_definition(size, values),
        Statement::BranchOnIndicators(op, branchloc) => {
            codegen_branch_on_indicators(op, branchloc, cur_addr, label_table)
        }
        Statement::BranchOnRegisters(op, reg, branchloc) => {
            codegen_branch_on_registers(op, reg, branchloc, cur_addr, label_table)
        }
        Statement::ShortValueImmediate(op, reg, value) => {
            codegen_short_value_immediate(op, reg, *value)
        }
        Statement::SingleOperand(op, addr_syl, mask) => {
            codegen_single_operand(op, addr_syl, mask, cur_addr, label_table)
        }
        Statement::Generic(op) => codegen_generic(op),
        Statement::DoubleOperand(op, reg, addr_syl, mask) => {
            codegen_double_operand(op, reg, addr_syl, mask, cur_addr, label_table)
        }
        Statement::ShiftShort(op, reg, dist) => codegen_shift_short(op, reg, *dist),
        Statement::ShiftLong(op, reg, dist) => codegen_shift_long(op, reg, *dist),
        Statement::InputOutput(op, data_addr_syl, chan_expr) => {
            codegen_input_output(op, data_addr_syl, chan_expr, cur_addr, label_table)
        }
        Statement::InputOutputLoad(buffer_addr_syl, chan_expr, range_addr_syl) => {
            codegen_input_output_load(
                buffer_addr_syl,
                chan_expr,
                range_addr_syl,
                cur_addr,
                label_table,
            )
        }
    }
}

/// Compute location of a brancloc field, returning the optional extra words as well
pub fn get_branch_location_field_value(
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
                Err(_) => return Err(AssemblerErrorKind::BranchAddressOutOfRange(addr)),
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
                Err(_) => {
                    return Err(AssemblerErrorKind::BranchLongDisplacementOutOfRange(
                        displacement,
                    ))
                }
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
                return Err(AssemblerErrorKind::BranchShortDisplacementOutOfRange(
                    displacement,
                ));
            }
            if displacement == 0 || displacement == 1 {
                return Err(AssemblerErrorKind::BranchShortDisplacementMustNotBe0Or1);
            }

            // Fit displacement in 8 bits TODO find way to make this more elegant
            let displacement_i8: i8 = match displacement.try_into() {
                Ok(res) => res,
                Err(_) => {
                    return Err(AssemblerErrorKind::BranchLongDisplacementOutOfRange(
                        displacement,
                    ))
                }
            };

            // Fit displacement in 7 bits
            let displacement_i7 = match i7::new(displacement_i8) {
                Some(res) => res,
                None => {
                    return Err(AssemblerErrorKind::BranchLongDisplacementOutOfRange(
                        displacement,
                    ))
                }
            };

            (displacement_i7, vec![])
        }
    };

    Ok((value, extra_words))
}

/// Resolve address expression to an absolute address in the u64 address space
pub fn resolve_address_expression(
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

pub fn get_data_register_value(reg: &DataRegister) -> u3 {
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

/// Returns u16 containing a two's complement encoded relative number
fn twos_complement_u16(input: i16) -> u16 {
    let bytes = input.to_be_bytes();
    u16::from_be_bytes(bytes)
}

pub fn get_maskword_value(mask: i128) -> Result<u16, AssemblerErrorKind> {
    // Maskword cannot be negative
    if mask < 0 {
        return Err(AssemblerErrorKind::MaskWordOutOfRange(mask));
    }

    match TryInto::<u16>::try_into(mask) {
        Ok(val) => Ok(val),
        Err(_) => Err(AssemblerErrorKind::MaskWordOutOfRange(mask)),
    }
}

pub fn get_generic_register_value(reg: &Register) -> u3 {
    match reg {
        Register::Base(reg) => get_base_register_value(&reg),
        Register::Data(reg) => get_data_register_value(&reg),
        Register::ModeControl(reg) => get_mode_control_register_value(&reg),
    }
}

pub fn get_base_register_value(reg: &BaseRegister) -> u3 {
    match reg {
        BaseRegister::B1 => u3!(1),
        BaseRegister::B2 => u3!(2),
        BaseRegister::B3 => u3!(3),
        BaseRegister::B4 => u3!(4),
        BaseRegister::B5 => u3!(5),
        BaseRegister::B6 => u3!(6),
        BaseRegister::B7 => u3!(7),
    }
}

pub fn get_mode_control_register_value(reg: &ModeControlRegister) -> u3 {
    match reg {
        ModeControlRegister::M1 => u3!(1),
        ModeControlRegister::M2 => u3!(2),
        ModeControlRegister::M3 => u3!(3),
        ModeControlRegister::M4 => u3!(4),
        ModeControlRegister::M5 => u3!(5),
        ModeControlRegister::M6 => u3!(6),
        ModeControlRegister::M7 => u3!(7),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
                u64::MAX,
            ),
        ];

        let label_table = HashMap::from([
            ("LABEL1".to_owned(), 1000),
            ("albatross".to_owned(), u64::MAX),
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
    pub fn get_data_register_value_succ() {
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
