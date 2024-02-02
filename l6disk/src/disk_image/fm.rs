use std::vec;

#[derive(Debug, Clone)]
pub struct FMByte {
    pub data: u8,
    pub clock: u8,
}

#[derive(Debug, Clone)]
pub struct FMBytes {
    bytes: Vec<FMByte>,
}

impl FMBytes {
    pub fn new() -> Self {
        Self { bytes: vec![] }
    }

    pub fn add_bytes(&mut self, new_bytes: &[u8]) {
        for new_byte in new_bytes {
            self.bytes.push(FMByte {
                data: *new_byte,
                clock: 0xFF,
            });
        }
    }

    pub fn add_fm_byte(&mut self, new_fm_byte: &FMByte) {
        self.bytes.push(new_fm_byte.clone());
    }

    pub fn get_data_bytes(&self) -> Vec<u8> {
        let mut data_bytes: Vec<u8> = vec![];

        for fm_byte in self.bytes.iter() {
            data_bytes.push(fm_byte.data);
        }

        data_bytes
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut encoded_bytes: Vec<u8> = vec![];

        for fm_byte in self.bytes.iter() {
            let mut byte_data: u8 = fm_byte.data;
            let mut byte_clock: u8 = fm_byte.clock;
            let mut byte_fm: u16 = 0;

            // Shift each bit into number
            for _ in 0..8 {
                // Add clock bit
                byte_fm <<= 1;
                byte_fm |= (byte_clock >> 7) as u16;

                // // Add data bit
                byte_fm <<= 1;
                byte_fm |= (byte_data >> 7) as u16;

                // Next bit
                byte_data <<= 1;
                byte_clock <<= 1;
            }

            // Add byte to result
            encoded_bytes.append(&mut byte_fm.to_be_bytes().to_vec());
        }

        encoded_bytes
    }

    pub fn append(&mut self, other: &mut FMBytes) {
        self.bytes.append(&mut other.bytes);
    }

    pub fn fm_len(&self) -> usize {
        self.bytes.len() * 2 // FM length is double byte length
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fm_enc() {
        let mut data = FMBytes::new();

        let byte = FMByte {
            data: 0b11111100,
            clock: 0b11010111,
        };

        data.add_fm_byte(&byte);

        assert_eq!(data.encode(), [0b11110111, 0b01111010]);
    }
}
