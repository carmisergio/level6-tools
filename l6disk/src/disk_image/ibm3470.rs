use crc::{Crc, CRC_16_IBM_3740};

// In-module imports
use super::convert::{Sector, Track};
use super::disk_parameters::DiskParameters;
use super::encode::calc_interleave_map;
use super::fm::{FMByte, FMBytes};

// Level6 Disk format Address Marks
#[non_exhaustive]
struct IBM3470AddressMark;
impl IBM3470AddressMark {
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

struct IBM3470Gaps;
impl IBM3470Gaps {
    pub const FILL_BYTE: u8 = 0xFF;
    pub const GAP1_LEN: usize = 40;
    pub const GAP2_LEN: usize = 26;
    pub const GAP3_LEN: usize = 11;
    pub const GAP4_LEN: usize = 27;
    pub const GAP5_LESS: usize = 100; // Bytes of GAP5 to omit
}

// Encode one track to Level6 format
pub fn encode_track(
    sectors: &[Sector],
    disk_parameters: &DiskParameters,
    cyl_n: u16,
    side_n: u16,
) -> Result<Track, String> {
    // Validate disk parameters
    check_disk_parameters(disk_parameters)?;

    // Compute sector interleave map
    let interleave_map = calc_interleave_map(
        disk_parameters.sectors_per_track,
        disk_parameters.sector_interleave,
    );

    // Track data buffer
    let mut track = FMBytes::new();

    // Encode track header
    track.append(&mut encode_track_header());

    // Encode sectors
    for phys_sec_n in 0..disk_parameters.sectors_per_track {
        // Compute interleave
        let logical_sec_n = interleave_map[phys_sec_n as usize];

        // Encode single sector
        track.append(&mut encode_sector(
            &sectors[logical_sec_n as usize],
            cyl_n as u8,
            logical_sec_n as u8 + 1, // Sector numbers start from 1
            side_n as u8,
        ));
    }

    // Fill remaining part of track
    let remaining_bytes: isize = (((60000) * disk_parameters.cell_rate as u64)
        / (disk_parameters.rpm as u64 * 8)) as isize
        - track.fm_len() as isize
        - IBM3470Gaps::GAP5_LESS as isize;

    // Check if sectors can fit on disk
    if remaining_bytes < 0 {
        return Err(format!("Track too large"));
    }

    // GAP5
    track.add_bytes(&vec![IBM3470Gaps::FILL_BYTE; remaining_bytes as usize / 2]); // divided by 2 because 1 data byte = 2 fm bytes

    Ok(track.encode())
}

fn check_disk_parameters(disk_parameters: &DiskParameters) -> Result<(), String> {
    // Check sector size
    if disk_parameters.bytes_per_sector > 255 {
        return Err(format!(
            "Sector size too large: {}",
            disk_parameters.bytes_per_sector
        ));
    };

    // Check number of sides
    if disk_parameters.n_sides > 255 {
        return Err(format!("Too many heads: {}", disk_parameters.n_sides));
    };

    // Check number of cylinders
    if disk_parameters.n_cylinders > 255 {
        return Err(format!(
            "Too many cylinders: {}",
            disk_parameters.n_cylinders
        ));
    };

    // Check sectors per track
    if disk_parameters.sectors_per_track > 255 {
        return Err(format!(
            "Too many sectors per track: {}",
            disk_parameters.sectors_per_track
        ));
    };

    Ok(())
}

fn encode_track_header() -> FMBytes {
    let mut data = FMBytes::new();

    // Pre-Index Gap (GAP1)
    data.add_bytes(&[IBM3470Gaps::FILL_BYTE; IBM3470Gaps::GAP1_LEN]);

    // AM4 Sync field (6 bytes)
    data.add_bytes(&[0x00; 6]);

    // Index address mark (AM4)
    data.add_fm_byte(&IBM3470AddressMark::IAM);

    // Post-Index address mark (GAP2)
    data.add_bytes(&[IBM3470Gaps::FILL_BYTE; IBM3470Gaps::GAP2_LEN]);

    data
}

fn encode_sector(sector: &Sector, track_n: u8, sector_n: u8, side_n: u8) -> FMBytes {
    let mut data = FMBytes::new();

    // Sector ID Sync field (6 bytes)
    data.add_bytes(&[0x00; 6]);

    // Sector ID field
    data.append(&mut encode_sector_header(
        sector.len() as u8,
        track_n,
        sector_n,
        side_n,
    ));

    // Identifier to Data Gap (GAP3)
    data.add_bytes(&[IBM3470Gaps::FILL_BYTE; IBM3470Gaps::GAP3_LEN]);

    // AM2 Sync field (6 bytes)
    data.add_bytes(&[0x00; 6]);

    // Sector data field
    data.append(&mut encode_sector_data(sector));

    // Intrer-sector Gap (GAP4)
    data.add_bytes(&[IBM3470Gaps::FILL_BYTE; IBM3470Gaps::GAP4_LEN]);

    data
}

fn encode_sector_header(sector_len: u8, track_n: u8, sector_n: u8, side_n: u8) -> FMBytes {
    let mut header = FMBytes::new();

    // Sector ID address mark (AM1)
    header.add_fm_byte(&IBM3470AddressMark::IDAM);

    // Sector ID fields
    // |-- TRACK_N --|--- RSU ---|- SECTOR_N -|--- RSU ---|
    // RSU: Reserved for software use
    header.add_bytes(&[track_n, side_n, sector_n, sector_len]);

    // Compuute crc
    let crc = Crc::<u16>::new(&CRC_16_IBM_3740);
    let mut digest = crc.digest();
    digest.update(&header.get_data_bytes());

    // Sector ID field EDC (CRC)
    header.add_bytes(&digest.finalize().to_be_bytes());
    // header.add_bytes(&[0x00, 0x00]);

    header
}

fn encode_sector_data(sector: &Sector) -> FMBytes {
    let mut data = FMBytes::new();

    // Sector Data field address mark (AM1)
    data.add_fm_byte(&IBM3470AddressMark::DAM);

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
