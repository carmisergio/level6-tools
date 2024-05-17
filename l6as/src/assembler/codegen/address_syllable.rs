use crate::{
    assembler::statements::{
        AddressSyllable, BRelativeAddress, BRelativeAddressMode, BaseRegister, ImmediateAddress,
        ImmediateAddressMode, IncDec, PRelativeAddress, Register,
    },
    logging::AssemblerErrorKind,
};
use std::collections::HashMap;

use super::common::{
    get_base_register_value, get_generic_register_value, resolve_address_expression,
};

use bit_struct::*;

use crate::assembler::statements::{AddressExpression, DataRegister};

// Address Syllable field structure
bit_struct! {
    pub struct AddressSyllableField(u7) {
        addr_mod: u3,
        ind_addr_bit: u1,
        reg: u3
    }
}

/// Encode an address syllable
pub fn get_address_syllable_field_value(
    addr_syl: &AddressSyllable,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<(u7, Vec<u16>), AssemblerErrorKind> {
    let (field, extra_words) = match addr_syl {
        AddressSyllable::RegisterAddressing(reg) => encode_addr_syl_register_addressing(reg)?,
        AddressSyllable::ImmediateOperand(val) => encode_addr_syl_immediate_operand(*val)?,
        AddressSyllable::ImmediateAddressing(imm_addr_mode) => {
            encode_addr_syl_immediate_addressing(imm_addr_mode, cur_addr, label_table)?
        }
        AddressSyllable::BRelative(brel_addr_mode) => {
            encode_addr_syl_brelative_addressing(brel_addr_mode)?
        }
        AddressSyllable::PRelative(prel_addr_mode) => {
            encode_addr_syl_prelative_addressing(prel_addr_mode, cur_addr, label_table)?
        }
    };
    Ok((field.raw(), extra_words))
}

fn encode_addr_syl_register_addressing(
    reg: &Register,
) -> Result<(AddressSyllableField, Vec<u16>), AssemblerErrorKind> {
    let field = AddressSyllableField::new(u3!(5), u1!(0), get_generic_register_value(reg));
    Ok((field, vec![]))
}

fn get_base_register_value_indexed(reg: &BaseRegister) -> Result<u3, AssemblerErrorKind> {
    Ok(match reg {
        BaseRegister::B1 => u3!(1),
        BaseRegister::B2 => u3!(2),
        BaseRegister::B3 => u3!(3),
        _ => return Err(AssemblerErrorKind::InvalidBaseRegisterAddrSyl(reg.clone())),
    })
}

fn encode_addr_syl_immediate_operand(
    val: i128,
) -> Result<(AddressSyllableField, Vec<u16>), AssemblerErrorKind> {
    let field = AddressSyllableField::new(u3!(7), u1!(0), u3!(0));
    Ok((field, vec![get_immediate_operand_extra_word(val)?]))
}

fn get_immediate_operand_extra_word(val: i128) -> Result<u16, AssemblerErrorKind> {
    if val >= 0 {
        // Raw value into u8
        match TryInto::<u16>::try_into(val) {
            Ok(val) => Ok(val),
            Err(_) => Err(AssemblerErrorKind::ImmediateValueOutOfRange(val)),
        }
    } else {
        // Two's complemented i8 into u8
        match TryInto::<u16>::try_into(val) {
            Ok(val) => Ok(u16::from_be_bytes(val.to_be_bytes())),
            Err(_) => Err(AssemblerErrorKind::ImmediateValueOutOfRange(val)),
        }
    }
}
fn encode_addr_syl_immediate_addressing(
    imm_addr: &ImmediateAddressMode,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<(AddressSyllableField, Vec<u16>), AssemblerErrorKind> {
    let (ind_addr_bit, (addr_modif, extra_words)) = match imm_addr {
        ImmediateAddressMode::Direct(imm_addr) => (
            u1!(0),
            get_addr_syl_immediate_addressing_addr_modif_field(imm_addr, cur_addr, label_table)?,
        ),
        ImmediateAddressMode::Indirect(imm_addr) => (
            u1!(1),
            get_addr_syl_immediate_addressing_addr_modif_field(imm_addr, cur_addr, label_table)?,
        ),
    };

    let field = AddressSyllableField::new(addr_modif, ind_addr_bit, u3!(0));
    Ok((field, extra_words))
}

fn get_addr_syl_immediate_addressing_addr_modif_field(
    imm_addr: &ImmediateAddress,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<(u3, Vec<u16>), AssemblerErrorKind> {
    Ok(match imm_addr {
        ImmediateAddress::Simple(ae) => (
            u3!(0),
            vec![get_immediate_address_extra_word(ae, cur_addr, label_table)?],
        ),
        ImmediateAddress::Indexed(ae, reg) => (
            get_index_register_addr_modif(reg)?,
            vec![get_immediate_address_extra_word(ae, cur_addr, label_table)?],
        ),
    })
}

fn get_immediate_address_extra_word(
    ae: &AddressExpression,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<u16, AssemblerErrorKind> {
    let addr = resolve_address_expression(ae, cur_addr, label_table)?;

    // Verify address fits in field
    match TryInto::<u16>::try_into(addr) {
        Ok(addr) => Ok(addr),
        Err(_) => Err(AssemblerErrorKind::ImmediateAddressOutOfRange(addr)),
    }
}

fn get_index_register_addr_modif(reg: &DataRegister) -> Result<u3, AssemblerErrorKind> {
    Ok(match reg {
        DataRegister::R1 => u3!(1),
        DataRegister::R2 => u3!(2),
        DataRegister::R3 => u3!(3),
        _ => return Err(AssemblerErrorKind::InvalidIndexRegister(reg.clone())),
    })
}

fn encode_addr_syl_brelative_addressing(
    brel_addr_mode: &BRelativeAddressMode,
) -> Result<(AddressSyllableField, Vec<u16>), AssemblerErrorKind> {
    match brel_addr_mode {
        BRelativeAddressMode::Direct(brel_addr) => {
            let (addr_mod, reg, extra_words) =
                get_addr_syl_brelative_addressing_direct_indirect(brel_addr)?;
            Ok((
                AddressSyllableField::new(addr_mod, u1!(0), reg),
                extra_words,
            ))
        }
        BRelativeAddressMode::Indirect(brel_addr) => {
            let (addr_mod, reg, extra_words) =
                get_addr_syl_brelative_addressing_direct_indirect(brel_addr)?;
            Ok((
                AddressSyllableField::new(addr_mod, u1!(1), reg),
                extra_words,
            ))
        }
        BRelativeAddressMode::PushPop(reg, incdec) => Ok((
            AddressSyllableField::new(
                match incdec {
                    IncDec::Decrement => u3!(6),
                    IncDec::Increment => u3!(7),
                },
                u1!(0),
                get_base_register_value(reg),
            ),
            vec![],
        )),
        BRelativeAddressMode::IncDecIndexed(reg, ind_reg, incdec) => Ok((
            AddressSyllableField::new(
                get_index_register_addr_modif(ind_reg)? + u3!(4), // Starts at 5
                u1!(1),
                match incdec {
                    IncDec::Decrement => get_base_register_value_indexed(reg)?,
                    IncDec::Increment => get_base_register_value_indexed(reg)? + u3!(4),
                },
            ),
            vec![],
        )),
    }
}

fn get_addr_syl_brelative_addressing_direct_indirect(
    brel_addr: &BRelativeAddress,
) -> Result<(u3, u3, Vec<u16>), AssemblerErrorKind> {
    Ok(match brel_addr {
        BRelativeAddress::Simple(reg) => (u3!(0), get_base_register_value(reg), vec![]),
        BRelativeAddress::Indexed(reg, ind_reg) => (
            get_index_register_addr_modif(ind_reg)?,
            get_base_register_value(reg),
            vec![],
        ),
        BRelativeAddress::Displacement(reg, disp) => (
            u3!(4),
            get_base_register_value(reg),
            vec![get_displacement_extra_word(*disp)?],
        ),
    })
}

fn get_displacement_extra_word(disp: i128) -> Result<u16, AssemblerErrorKind> {
    // Verify address fits in field
    match TryInto::<i16>::try_into(disp) {
        Ok(disp) => Ok(u16::from_be_bytes(disp.to_be_bytes())),
        Err(_) => Err(AssemblerErrorKind::DisplacementOutOfRange(disp)),
    }
}

fn encode_addr_syl_prelative_addressing(
    prel_addr: &PRelativeAddress,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<(AddressSyllableField, Vec<u16>), AssemblerErrorKind> {
    let (ind_addr_bit, extra_word) = match prel_addr {
        PRelativeAddress::Direct(ae) => (
            u1!(0),
            get_prelative_address_extra_word(ae, cur_addr, label_table)?,
        ),
        PRelativeAddress::Indirect(ae) => (
            u1!(1),
            get_prelative_address_extra_word(ae, cur_addr, label_table)?,
        ),
    };

    let field = AddressSyllableField::new(u3!(4), ind_addr_bit, u3!(0));
    Ok((field, vec![extra_word]))
}

fn get_prelative_address_extra_word(
    ae: &AddressExpression,
    cur_addr: u64,
    label_table: &HashMap<String, u64>,
) -> Result<u16, AssemblerErrorKind> {
    let addr = resolve_address_expression(ae, cur_addr, label_table)?;

    let disp = addr as i128 - cur_addr as i128;

    // Verify address fits in field
    match TryInto::<i16>::try_into(disp) {
        Ok(disp) => Ok(u16::from_be_bytes(disp.to_be_bytes())),
        Err(_) => Err(AssemblerErrorKind::DisplacementOutOfRange(disp)),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn get_address_syllable_field_value_succ() {
        let tests = [
            (
                AddressSyllable::RegisterAddressing(Register::Data(DataRegister::R1)),
                0,
                u7!(0b1010001),
                vec![],
            ),
            (
                AddressSyllable::RegisterAddressing(Register::Base(BaseRegister::B2)),
                0,
                u7!(0b1010010),
                vec![],
            ),
            (
                AddressSyllable::ImmediateOperand(0x1234),
                0,
                u7!(0b1110000),
                vec![0x1234],
            ),
            (
                AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Direct(
                    ImmediateAddress::Simple(AddressExpression::Immediate(0x1234)),
                )),
                0,
                u7!(0b0000000),
                vec![0x1234],
            ),
            (
                AddressSyllable::ImmediateAddressing(ImmediateAddressMode::Indirect(
                    ImmediateAddress::Indexed(
                        AddressExpression::Immediate(0x4000),
                        DataRegister::R3,
                    ),
                )),
                100,
                u7!(0b0111000),
                vec![0x4000],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::Direct(BRelativeAddress::Simple(
                    BaseRegister::B1,
                ))),
                0,
                u7!(0b0000001),
                vec![],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::Indirect(
                    BRelativeAddress::Simple(BaseRegister::B2),
                )),
                0,
                u7!(0b0001010),
                vec![],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::Direct(
                    BRelativeAddress::Indexed(BaseRegister::B3, DataRegister::R1),
                )),
                0,
                u7!(0b0010011),
                vec![],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::Indirect(
                    BRelativeAddress::Indexed(BaseRegister::B4, DataRegister::R2),
                )),
                0,
                u7!(0b0101100),
                vec![],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::Direct(
                    BRelativeAddress::Displacement(BaseRegister::B5, -10),
                )),
                0,
                u7!(0b1000101),
                vec![u16::from_be_bytes((-10 as i16).to_be_bytes())],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::Indirect(
                    BRelativeAddress::Displacement(BaseRegister::B6, 0x2000),
                )),
                0,
                u7!(0b1001110),
                vec![0x2000],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::PushPop(
                    BaseRegister::B7,
                    IncDec::Increment,
                )),
                0,
                u7!(0b1110111),
                vec![],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::PushPop(
                    BaseRegister::B1,
                    IncDec::Decrement,
                )),
                0,
                u7!(0b1100001),
                vec![],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::IncDecIndexed(
                    BaseRegister::B2,
                    DataRegister::R3,
                    IncDec::Increment,
                )),
                0,
                u7!(0b1111110),
                vec![],
            ),
            (
                AddressSyllable::BRelative(BRelativeAddressMode::IncDecIndexed(
                    BaseRegister::B3,
                    DataRegister::R1,
                    IncDec::Decrement,
                )),
                0,
                u7!(0b1011011),
                vec![],
            ),
            (
                AddressSyllable::PRelative(PRelativeAddress::Direct(AddressExpression::Immediate(
                    0x1234,
                ))),
                0x1000,
                u7!(0b1000000),
                vec![0x0234],
            ),
            (
                AddressSyllable::PRelative(PRelativeAddress::Indirect(
                    AddressExpression::WordDisplacement(-100),
                )),
                0x3834,
                u7!(0b1001000),
                vec![u16::from_be_bytes((-100 as i16).to_be_bytes())],
            ),
        ];

        let label_table: HashMap<String, u64> = HashMap::new();

        for (input, cur_addr, field_exp, ew_exp) in tests {
            let (field, ew) =
                get_address_syllable_field_value(&input, cur_addr, &label_table).unwrap();

            assert_eq!(field, field_exp);
            assert_eq!(ew, ew_exp);
        }
    }
}
