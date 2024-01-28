// In-module imports
use super::convert::{Sector, Track};
use super::disk_parameters::DiskParameters;
use super::encode::calc_interleave_map;

// Level6 Disk format Address Marks
#[non_exhaustive]
struct Level6AddressMark;
impl Level6AddressMark {
    pub const AM1: [u8; 2] = [0b11110101, 0b01111110];
    pub const AM2: [u8; 2] = [0b11110101, 0b01101111];
    // pub const AM3: [u8; 2] = [0b11110101, 0b01101010];
    pub const AM4: [u8; 2] = [0b11110111, 0b01111010];
}

struct Level6Gaps;
impl Level6Gaps {
    pub const FILL_BYTE: u8 = 0x00;
    // pub const GAP1_LEN: usize = 46;
    // pub const GAP2_LEN: usize = 32;
    // pub const GAP3_LEN: usize = 17;
    pub const GAP1_LEN: usize = 1;
    pub const GAP2_LEN: usize = 2;
    pub const GAP3_LEN: usize = 3;
}

// Encode one track to Level6 format
pub fn encode_track_level6(
    sectors: &[Sector],
    disk_parameters: &DiskParameters,
    cyl_n: u16,
    side_n: u16,
) -> Result<Track, String> {
    // Check if side number is valid
    if side_n > 0 {
        return Err(format!(
            "Invalid side number for Level6 track format: {}",
            side_n
        ));
    }

    // Compute sector interleave map
    let interleave_map = calc_interleave_map(
        disk_parameters.sectors_per_track,
        disk_parameters.sector_interleave,
    );

    // Track data buffer
    let mut track: Track = vec![];

    // Encode track header
    track.append(&mut level6_encode_track_header());

    // Encode sectors
    for phys_sec_n in 0..disk_parameters.sectors_per_track {
        // Compute interleave
        let logical_sec_n = interleave_map[phys_sec_n as usize];

        println!(
            "Encoding sector: cyl={}, phys={}, log={}",
            cyl_n, phys_sec_n, logical_sec_n
        );

        // Encode single sector
        track.append(&mut level6_encode_sector(
            &sectors[logical_sec_n as usize],
            cyl_n as u8,
            logical_sec_n as u8 + 1, // Sector numbers start from 1
        ));
    }

    Ok(track)
}

fn level6_encode_track_header() -> Vec<u8> {
    let mut header: Vec<u8> = vec![];

    // Pre-Index Gap (GAP1)
    header.append(&mut encode_fm(&vec![
        Level6Gaps::FILL_BYTE;
        Level6Gaps::GAP1_LEN
    ]));

    // Index address mark (AM4)
    header.append(&mut Level6AddressMark::AM4.to_vec());

    // Post-Index address mark (GAP2)
    header.append(&mut encode_fm(&vec![
        Level6Gaps::FILL_BYTE;
        Level6Gaps::GAP2_LEN
    ]));

    header
}

fn level6_encode_sector(sector: &Sector, track_n: u8, sector_n: u8) -> Vec<u8> {
    let mut sec_data: Vec<u8> = vec![];

    // Sector ID address mark (AM1)
    sec_data.append(&mut Level6AddressMark::AM1.to_vec());

    // Sector ID fields
    // |-- TRACK_N --|--- RSU ---|- SECTOR_N -|--- RSU ---|
    // RSU: Reserved for software use
    sec_data.append(&mut encode_fm(&vec![track_n, 0x00, sector_n, 0x00]));

    // Sector ID EDC TODO implement
    sec_data.append(&mut encode_fm(&vec![0xFF, 0xFF]));

    // Identifier to Data Gap (GAP3)
    sec_data.append(&mut encode_fm(&vec![
        Level6Gaps::FILL_BYTE;
        Level6Gaps::GAP3_LEN
    ]));

    // Data Field address mark (AM2)
    sec_data.append(&mut Level6AddressMark::AM2.to_vec());

    // Sector data
    sec_data.append(&mut encode_fm(sector));

    // Data Field EDC TODO implement
    sec_data.append(&mut encode_fm(&vec![0xFF, 0xFF]));

    // Intrer-sector Gap (GAP2)
    sec_data.append(&mut encode_fm(&vec![
        Level6Gaps::FILL_BYTE;
        Level6Gaps::GAP2_LEN
    ]));

    sec_data
}

// Encode bytes to FM
// Interleaves the bits of the provided bytes with clock
fn encode_fm(data: &Vec<u8>) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];

    for byte in data.iter() {
        let mut this_byte: u8 = byte.clone();
        let mut byte_fm: u16 = 0;

        // Shift each bit into number
        for _ in 0..8 {
            // Add clock bit
            byte_fm <<= 1;
            byte_fm |= 1;

            // // Add bit to right
            byte_fm <<= 1;
            byte_fm |= (this_byte >> 7) as u16;

            // Next bit
            this_byte <<= 1;

            // println!("{:#b}", this_byte);
        }

        // Add byte to result
        encoded.append(&mut byte_fm.to_be_bytes().to_vec());
    }

    encoded
}
