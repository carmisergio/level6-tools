use crate::{assembler::statements::GenericOpCode, logging::AssemblerErrorKind};

use bit_struct::*;

bit_struct! {
    pub struct GenericInstructionWord(u16) {
        header: u8,
        op: u8,
    }
}

/// Generaete code for a Generic instruction
pub fn codegen_generic(op: &GenericOpCode) -> Result<Vec<u16>, AssemblerErrorKind> {
    // Build instruction word
    let inst_word = GenericInstructionWord::new(0b00000000, get_generic_op_value(op));

    Ok(vec![inst_word.raw()])
}

fn get_generic_op_value(op: &GenericOpCode) -> u8 {
    match op {
        GenericOpCode::HLT => 0b00000000,
        GenericOpCode::MCL => 0b00000001,
        GenericOpCode::RTT => 0b00000011,
        GenericOpCode::RTCN => 0b00000100,
        GenericOpCode::RTCF => 0b00000101,
        GenericOpCode::WDTN => 0b00000110,
        GenericOpCode::WDTF => 0b00000111,
        GenericOpCode::BRK => 0b00000010,
        GenericOpCode::MMM => 0b00001000,
        GenericOpCode::ASD => 0b00001010,
        GenericOpCode::VLD => 0b00001011,
        GenericOpCode::QOH => 0b01100000,
        GenericOpCode::QOT => 0b01100001,
        GenericOpCode::DQH => 0b01100010,
        GenericOpCode::DQA => 0b01100011,
        GenericOpCode::RSC => 0b00010001,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codegen_generic_instructions_succ() {
        let tests = [
            (GenericOpCode::HLT, vec![0b0000000000000000]),
            (GenericOpCode::MCL, vec![0b0000000000000001]),
            (GenericOpCode::RTT, vec![0b0000000000000011]),
            (GenericOpCode::RTCN, vec![0b0000000000000100]),
            (GenericOpCode::RTCF, vec![0b0000000000000101]),
            (GenericOpCode::WDTN, vec![0b0000000000000110]),
            (GenericOpCode::WDTF, vec![0b0000000000000111]),
            (GenericOpCode::BRK, vec![0b0000000000000010]),
            (GenericOpCode::MMM, vec![0b0000000000001000]),
            (GenericOpCode::ASD, vec![0b0000000000001010]),
            (GenericOpCode::VLD, vec![0b0000000000001011]),
            (GenericOpCode::QOH, vec![0b0000000001100000]),
            (GenericOpCode::QOT, vec![0b0000000001100001]),
            (GenericOpCode::DQH, vec![0b0000000001100010]),
            (GenericOpCode::DQA, vec![0b0000000001100011]),
            (GenericOpCode::RSC, vec![0b0000000000010001]),
        ];

        for (op, exp) in tests {
            assert_eq!(codegen_generic(&op).unwrap(), exp);
        }
    }
}
