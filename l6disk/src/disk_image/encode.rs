// In-module imports
pub use super::disk_parameters::DiskParameters;
pub use super::disk_parameters::DiskTrackFormat;
pub use super::errors::{EncodeError, EncodeErrorType};

#[derive(Debug)]
pub enum RawImageFormat {
    HFE,
}
#[derive(Debug)]
// Encoding options
pub struct EncodeOpts {
    pub ignore_errors: bool,
    pub disk_parameters: DiskParameters,
    pub out_file_format: RawImageFormat,
}

// Image encoding result
pub type EncodeResult = Result<Vec<u8>, EncodeError>;

// Floppy Data types
type Sector = Vec<u8>;
type Track = Vec<u8>;

// Convert data disk image to raw disk image
pub fn encode_disk_image(data_img: Vec<u8>, opts: EncodeOpts) -> EncodeResult {
    // Divide disk image into sectors
    let mut sectors = match divide_data_image_sectors(
        data_img,
        opts.disk_parameters.bytes_per_sector,
        opts.ignore_errors,
    ) {
        Ok(sectors) => sectors,
        Err(()) => return Err(EncodeError::new(EncodeErrorType::SectorDivision)),
    };

    // Check number of sectors
    let expected_sectors: usize =
        (opts.disk_parameters.sectors_per_track * opts.disk_parameters.n_tracks) as usize;
    if sectors.len() != expected_sectors {
        // Wrong number of sectors
        if opts.ignore_errors {
            if sectors.len() < expected_sectors {
                // Add sectors to fill remaining tracks
                sectors.append(&mut vec![
                    vec![
                        0;
                        opts.disk_parameters.bytes_per_sector as usize
                    ];
                    expected_sectors - sectors.len()
                ]);
            }
        } else {
            return Err(EncodeError::new(EncodeErrorType::SectorNumber));
        }
    }

    // Encode tracks to correct format
    let tracks = match encode_tracks(sectors, &opts.disk_parameters) {
        Ok(tracks) => tracks,
        Err(msg) => return Err(EncodeError::new(EncodeErrorType::TrackEncoding(msg))),
    };

    // for i in 0 as u16..9 as u16 {
    //     println!("{}: {}", i, calc_sector_interleave(i, 9, 4));
    // }

    // println!("{:?}", tracks);
    // println!("{} tracks generated", tracks.len());

    let generated_data: Vec<u8> = vec![b'a'; 100];

    Ok(generated_data)
}

type EncodeTracksResult = Result<Vec<Track>, &'static str>;

// Encode all tracks in the disk
fn encode_tracks(sectors: Vec<Sector>, disk_parameters: &DiskParameters) -> EncodeTracksResult {
    let mut tracks: Vec<Track> = vec![];

    // Encode each track of the disk
    for track_n in 0..disk_parameters.n_tracks {
        // Compute start sector of this track
        let start_sector: usize = track_n as usize * disk_parameters.sectors_per_track as usize;
        let end_sector: usize = start_sector + disk_parameters.sectors_per_track as usize;

        let track = match encode_track(
            &sectors[start_sector..end_sector].to_vec(),
            &disk_parameters,
            track_n,
        ) {
            Ok(track) => track,
            Err(msg) => return Err(msg),
        };

        // Add track to disk
        tracks.push(track);
    }

    Ok(tracks)
}

// Encode one track
fn encode_track(
    sectors: &Vec<Sector>,
    disk_parameters: &DiskParameters,
    track_n: u16,
) -> Result<Track, &'static str> {
    match disk_parameters.track_format {
        DiskTrackFormat::Level6 => encode_track_level6(sectors, &disk_parameters, track_n),
    }
}

// Encode one track to Level6 format
fn encode_track_level6(
    sectors: &Vec<Sector>,
    disk_parameters: &DiskParameters,
    track_n: u16,
) -> Result<Track, &'static str> {
    // Track data buffer
    let mut track: Track = vec![];

    // Encode track header
    track.append(&mut level6_encode_track_header());

    // Encode sectors
    for phys_sec_n in 0..disk_parameters.sectors_per_track {
        // Compute interleave
        let logical_sec_n = calc_sector_interleave(
            phys_sec_n,
            disk_parameters.sectors_per_track,
            disk_parameters.sector_interleave,
        );

        // Encode single sector
        track.append(&mut level6_encode_sector(
            &sectors[logical_sec_n as usize],
            track_n as u8,
            logical_sec_n as u8,
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
    sec_data.append(&mut encode_fm(&vec![track_n, 0x00, sector_n + 1, 0x00]));

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

// Get logical sector to be put in each physical sector for given interleave value
fn calc_sector_interleave(physical_sector: u16, n_sectors: u16, interleave: u16) -> u16 {
    (physical_sector % interleave) * ((n_sectors + 1) / interleave) + physical_sector / interleave
}

// Divide data image into vector of sectors with given size
fn divide_data_image_sectors(
    data_img: Vec<u8>,
    sector_size: u16,
    ignore_errors: bool,
) -> Result<Vec<Sector>, ()> {
    let mut sectors: Vec<Sector> = vec![];

    // Convert image to sectors
    let mut bytes_consumed: usize = 0;
    while bytes_consumed < data_img.len() {
        let remaining_bytes = data_img.len() - bytes_consumed;

        // Create new sector
        if remaining_bytes >= sector_size as usize {
            // Valid sector boundary
            sectors.push(data_img[bytes_consumed..bytes_consumed + sector_size as usize].to_vec());
        } else if ignore_errors {
            // Invalid sector boundary but we ignore errors

            // Fill remaining bytes with zeroes
            let mut this_sector: Sector = vec![0; sector_size as usize];
            this_sector[0..remaining_bytes]
                .copy_from_slice(&data_img[bytes_consumed..data_img.len()]);
            sectors.push(this_sector);
        } else {
            // Sector boundary error
            return Err(());
        }

        bytes_consumed += sector_size as usize;
    }

    Ok(sectors)
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
