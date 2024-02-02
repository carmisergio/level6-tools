use crc::{Crc, CRC_16_IBM_3740};

// In-module imports
use super::convert::{Sector, Track};
use super::disk_parameters::DiskParameters;
use super::encode::calc_interleave_map;
use super::fm::{FMByte, FMBytes};

// Level6 Disk format Address Marks
#[non_exhaustive]
struct Level6AddressMark;
impl Level6AddressMark {
    pub const IAM: FMByte = FMByte {
        data: 0xFC,
        clock: 0xD7,
    };
    pub const IDAM: FMByte = FMByte {
        data: 0xFE,
        clock: 0xC7,
    };
    pub const DAM: FMByte = FMByte {
        data: 0xFB,
        clock: 0xC7,
    };
}

struct Level6Gaps;
impl Level6Gaps {
    pub const FILL_BYTE: u8 = 0xFF;
    pub const GAP1_LEN: usize = 40;
    pub const GAP2_LEN: usize = 26;
    pub const GAP3_LEN: usize = 11;
    pub const GAP4_LEN: usize = 27;
    pub const GAP5_LESS: usize = 100; // Bytes of GAP5 to omit
}

// Encode one track to Level6 format
pub fn encode_track_level6(
    sectors: &[Sector],
    disk_parameters: &DiskParameters,
    cyl_n: u16,
    _side_n: u16,
) -> Result<Track, String> {
    // Check if side number is valid
    // if side_n > 0 {
    //     return Err(format!(
    //         "Invalid side number for Level6 track format: {}",
    //         side_n
    //     ));
    // }

    // Compute sector interleave map
    let interleave_map = calc_interleave_map(
        disk_parameters.sectors_per_track,
        disk_parameters.sector_interleave,
    );

    // Track data buffer
    let mut track = FMBytes::new();

    // Encode track header
    track.append(&mut level6_encode_track_header());

    // Encode sectors
    for phys_sec_n in 0..disk_parameters.sectors_per_track {
        // Compute interleave
        let logical_sec_n = interleave_map[phys_sec_n as usize];

        // Encode single sector
        track.append(&mut level6_encode_sector(
            &sectors[logical_sec_n as usize],
            cyl_n as u8,
            logical_sec_n as u8 + 1, // Sector numbers start from 1
        ));
    }

    // Fill remaining part of track
    let remaining_bytes = (((60000) * disk_parameters.bit_rate as u64)
        / (disk_parameters.rpm as u64 * 8)) as usize
        - track.fm_len()
        - Level6Gaps::GAP5_LESS;

    // GAP5
    track.add_bytes(&vec![Level6Gaps::FILL_BYTE; remaining_bytes / 2]); // divided by 2 because 1 data byte = 2 fm bytes

    Ok(track.encode())
}

fn level6_encode_track_header() -> FMBytes {
    let mut data = FMBytes::new();

    // Pre-Index Gap (GAP1)
    data.add_bytes(&[Level6Gaps::FILL_BYTE; Level6Gaps::GAP1_LEN]);

    // AM4 Sync field (6 bytes)
    data.add_bytes(&[0x00; 6]);

    // Index address mark (AM4)
    data.add_fm_byte(&Level6AddressMark::IAM);

    // Post-Index address mark (GAP2)
    data.add_bytes(&[Level6Gaps::FILL_BYTE; Level6Gaps::GAP2_LEN]);

    data
}

fn level6_encode_sector(sector: &Sector, track_n: u8, sector_n: u8) -> FMBytes {
    let mut data = FMBytes::new();

    // Sector ID Sync field (6 bytes)
    data.add_bytes(&[0x00; 6]);

    // Sector ID field
    data.append(&mut level6_encode_sector_header(
        sector.len() as u8,
        track_n,
        sector_n,
    ));

    // Identifier to Data Gap (GAP3)
    data.add_bytes(&[Level6Gaps::FILL_BYTE; Level6Gaps::GAP3_LEN]);

    // AM2 Sync field (6 bytes)
    data.add_bytes(&[0x00; 6]);

    // Sector data field
    data.append(&mut level6_encode_sector_data(sector));

    // Intrer-sector Gap (GAP4)
    data.add_bytes(&[Level6Gaps::FILL_BYTE; Level6Gaps::GAP4_LEN]);

    data
}

fn level6_encode_sector_header(sector_len: u8, track_n: u8, sector_n: u8) -> FMBytes {
    let mut header = FMBytes::new();

    // Sector ID address mark (AM1)
    header.add_fm_byte(&Level6AddressMark::IDAM);

    // Sector ID fields
    // |-- TRACK_N --|--- RSU ---|- SECTOR_N -|--- RSU ---|
    // RSU: Reserved for software use
    header.add_bytes(&[track_n, 0x00, sector_n, sector_len]);

    // Compuute crc
    let crc = Crc::<u16>::new(&CRC_16_IBM_3740);
    let mut digest = crc.digest();
    digest.update(&header.get_data_bytes());

    // Sector ID field EDC (CRC)
    header.add_bytes(&digest.finalize().to_be_bytes());
    // header.add_bytes(&[0x00, 0x00]);

    header
}

fn level6_encode_sector_data(sector: &Sector) -> FMBytes {
    let mut data = FMBytes::new();

    // Sector Data field address mark (AM1)
    data.add_fm_byte(&Level6AddressMark::DAM);

    // Sector data
    data.add_bytes(&sector);

    // Compuute crc
    let crc = Crc::<u16>::new(&CRC_16_IBM_3740);
    let mut digest = crc.digest();
    digest.update(&data.get_data_bytes());

    // Sector Data field EDC (CRC)
    data.add_bytes(&digest.finalize().to_be_bytes());

    data
}
