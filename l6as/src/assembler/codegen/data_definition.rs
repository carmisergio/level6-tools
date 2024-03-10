use crate::{assembler::statements::DataDefinitionSize, logging::AssemblerErrorKind};

/// Generaete data for a Data Definition directive
pub fn codegen_data_definition(
    size: &DataDefinitionSize,
    values: &Vec<i128>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    let mut res = vec![];

    for &val in values {
        // Encode value to words
        let mut words = match size {
            DataDefinitionSize::Byte => vec![],
            DataDefinitionSize::Word => encode_value_to_word(val)?,
            DataDefinitionSize::DoubleWord => vec![],
            DataDefinitionSize::QuadWord => vec![],
        };

        res.append(&mut words);
    }

    Ok(res)
}

// pub fn encode_value_to_byte(value: i128) -> Result<Vec<u16>, AssemblerErrorKind> {
//     let word = if value >= 0 {
//         // Raw value into u16
//         match TryInto::<u16>::try_into(value) {
//             Ok(val) => val,
//             Err(_) => return Err(AssemblerErrorKind::ShortImmediateValueOutOfRange(value)),
//         }
//     } else {
//         // Two's complemented i16 into u16
//         match TryInto::<i16>::try_into(value) {
//             Ok(val) => u16::from_be_bytes(val.to_be_bytes()),
//             Err(_) => return Err(AssemblerErrorKind::ShortImmediateValueOutOfRange(value)),
//         }
//     };

//     Ok(vec![word])
// }

pub fn encode_value_to_word(value: i128) -> Result<Vec<u16>, AssemblerErrorKind> {
    let word = if value >= 0 {
        // Raw value into u16
        match TryInto::<u16>::try_into(value) {
            Ok(val) => val,
            Err(_) => return Err(AssemblerErrorKind::ShortImmediateValueOutOfRange(value)),
        }
    } else {
        // Two's complemented i16 into u16
        match TryInto::<i16>::try_into(value) {
            Ok(val) => u16::from_be_bytes(val.to_be_bytes()),
            Err(_) => return Err(AssemblerErrorKind::ShortImmediateValueOutOfRange(value)),
        }
    };

    Ok(vec![word])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codegen_branch_on_indicators_instructions_succ() {
        let tests = [
            // ((DataDefinitionSize::Byte, vec![0x12, 0x89]), vec![0x1289]),
            // ((DataDefinitionSize::Byte, vec![0x23]), vec![0x2300]),
            (
                (DataDefinitionSize::Word, vec![0x1234, 0xABCD, 0, 12, -12]),
                vec![0x1234, 0xABCD, 0x0000, 12, 0xFFF4],
            ),
            // (
            //     (DataDefinitionSize::DoubleWord, vec![0x12345678, 0]),
            //     vec![0x1234, 0x5678, 0x0000, 0x0000],
            // ),
            // (
            //     (DataDefinitionSize::QuadWord, vec![0x12345678ABCD9876, 0]),
            //     vec![
            //         0x1234, 0x5678, 0xABCD, 0x9876, 0x0000, 0x0000, 0x0000, 0x0000,
            //     ],
            // ),
        ];

        for ((size, chunks), exp) in tests {
            assert_eq!(codegen_data_definition(&size, &chunks).unwrap(), exp);
        }
    }
}
