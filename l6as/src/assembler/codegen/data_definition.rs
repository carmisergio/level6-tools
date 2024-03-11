use crate::{assembler::statements::DataDefinitionSize, logging::AssemblerErrorKind};

/// Generaete data for a Data Definition directive
pub fn codegen_data_definition(
    size: &DataDefinitionSize,
    values: &Vec<i128>,
) -> Result<Vec<u16>, AssemblerErrorKind> {
    let words = match size {
        DataDefinitionSize::Byte => encode_values_to_byte(values)?,
        DataDefinitionSize::Word => encode_values_to_word(values)?,
        DataDefinitionSize::DoubleWord => encode_values_to_doubleword(values)?,
        DataDefinitionSize::QuadWord => encode_values_to_quadword(values)?,
    };

    Ok(words)
}

pub fn encode_values_to_byte(values: &[i128]) -> Result<Vec<u16>, AssemblerErrorKind> {
    let mut res = vec![];

    // Iterate over couples of bytes
    for start in 0..(values.len() + 1) / 2 {
        // Compute actual start index
        let start = start * 2;

        // First byte
        let first = value_to_u8(values[start])?;

        // Second byte
        let second = if start + 1 < values.len() {
            value_to_u8(values[start + 1])?
        } else {
            0x00
        };

        // Fit two bytes into 16-bit word
        res.push(u16::from_be_bytes([first, second]));
    }

    Ok(res)
}

pub fn value_to_u8(value: i128) -> Result<u8, AssemblerErrorKind> {
    if value >= 0 {
        // Raw value into u16
        match TryInto::<u8>::try_into(value) {
            Ok(val) => Ok(val),
            Err(_) => return Err(AssemblerErrorKind::DataDefinitionValueOutOfRange(value)),
        }
    } else {
        // Two's complemented i16 into u16
        match TryInto::<i8>::try_into(value) {
            Ok(val) => Ok(u8::from_be_bytes(val.to_be_bytes())),
            Err(_) => return Err(AssemblerErrorKind::DataDefinitionValueOutOfRange(value)),
        }
    }
}

fn encode_values_to_word(values: &[i128]) -> Result<Vec<u16>, AssemblerErrorKind> {
    let mut res = vec![];

    for &value in values {
        let word = if value >= 0 {
            // Raw value into u16
            match TryInto::<u16>::try_into(value) {
                Ok(val) => val,
                Err(_) => return Err(AssemblerErrorKind::DataDefinitionValueOutOfRange(value)),
            }
        } else {
            // Two's complemented i16 into u16
            match TryInto::<i16>::try_into(value) {
                Ok(val) => u16::from_be_bytes(val.to_be_bytes()),
                Err(_) => return Err(AssemblerErrorKind::DataDefinitionValueOutOfRange(value)),
            }
        };

        res.push(word);
    }

    Ok(res)
}

fn encode_values_to_doubleword(values: &[i128]) -> Result<Vec<u16>, AssemblerErrorKind> {
    let mut res = vec![];

    for &value in values {
        let word = if value >= 0 {
            // Raw value into u32 (double word)
            match TryInto::<u32>::try_into(value) {
                Ok(val) => val,
                Err(_) => return Err(AssemblerErrorKind::DataDefinitionValueOutOfRange(value)),
            }
        } else {
            // Two's complemented i32 into u32
            match TryInto::<i32>::try_into(value) {
                Ok(val) => u32::from_be_bytes(val.to_be_bytes()),
                Err(_) => return Err(AssemblerErrorKind::DataDefinitionValueOutOfRange(value)),
            }
        };

        // Get bytes from u32
        let bytes = word.to_be_bytes();

        // Pack bytes into two 16-bit words
        res.extend_from_slice(&[
            u16::from_be_bytes([bytes[0], bytes[1]]),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ]);
    }

    Ok(res)
}

fn encode_values_to_quadword(values: &[i128]) -> Result<Vec<u16>, AssemblerErrorKind> {
    let mut res = vec![];

    for &value in values {
        let word = if value >= 0 {
            // Raw value into u32 (double word)
            match TryInto::<u64>::try_into(value) {
                Ok(val) => val,
                Err(_) => return Err(AssemblerErrorKind::DataDefinitionValueOutOfRange(value)),
            }
        } else {
            // Two's complemented i32 into u32
            match TryInto::<i64>::try_into(value) {
                Ok(val) => u64::from_be_bytes(val.to_be_bytes()),
                Err(_) => return Err(AssemblerErrorKind::DataDefinitionValueOutOfRange(value)),
            }
        };

        // Get bytes from u64
        let bytes = word.to_be_bytes();

        // Pack bytes into four 16-bit words
        res.extend_from_slice(&[
            u16::from_be_bytes([bytes[0], bytes[1]]),
            u16::from_be_bytes([bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            u16::from_be_bytes([bytes[6], bytes[7]]),
        ]);
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codegen_branch_on_indicators_instructions_succ() {
        let tests = [
            ((DataDefinitionSize::Byte, vec![0x12, 0x89]), vec![0x1289]),
            ((DataDefinitionSize::Byte, vec![0x23]), vec![0x2300]),
            (
                (DataDefinitionSize::Byte, vec![0x00, 0xFF, 0x12]),
                vec![0x00FF, 0x1200],
            ),
            (
                (DataDefinitionSize::Word, vec![0x1234, 0xABCD, 0, 12, -12]),
                vec![0x1234, 0xABCD, 0x0000, 12, 0xFFF4],
            ),
            (
                (DataDefinitionSize::DoubleWord, vec![0x12345678, 0]),
                vec![0x1234, 0x5678, 0x0000, 0x0000],
            ),
            (
                (DataDefinitionSize::QuadWord, vec![0x12345678ABCD9876, 0]),
                vec![
                    0x1234, 0x5678, 0xABCD, 0x9876, 0x0000, 0x0000, 0x0000, 0x0000,
                ],
            ),
        ];

        for ((size, chunks), exp) in tests {
            assert_eq!(codegen_data_definition(&size, &chunks).unwrap(), exp);
        }
    }
}
