// In-module imports
use super::convert::Cylinder;
use super::disk_parameters::{DiskParameters, DiskTrackFormat};

const HFE_BLOCK_SIZE: usize = 512;
const HFE_PAD_VALUE: u8 = 0x00;

// Create HFE file from arary of encoded cylinders
pub fn make_hfe_file(
    encoded_cylinders: &Vec<Cylinder>,
    disk_parameters: &DiskParameters,
) -> Result<Vec<u8>, String> {
    // Check number of sides
    if disk_parameters.n_sides > 2 {
        return Err(format!(
            "Too many sides for HFE file: {}",
            disk_parameters.n_sides
        ));
    }

    // Encode track data and track LUT
    let mut track_offset_lut = HFETrackOffsetLUT::new();
    let mut track_data: Vec<u8> = vec![];

    // For each cylinder
    let mut used_blocks: u16 = 3; // File header (1 block) + Track offset lut (2 blocks)
    for cylinder in encoded_cylinders {
        // Check if track is too big to fit in HFE file
        if cylinder[0].len() as u16 > u16::MAX / 4 {
            return Err(format!("Track too big for HFE file",));
        }

        // Add entry to track entries
        track_offset_lut.add_track(used_blocks, cylinder[0].len() as u16 * 4); // * 2 and * 2 again because of the weird hfe encoding to keep space for 2 sides

        // Add track to track data
        let (mut this_track, n_blocks) = pack_track(&cylinder);

        track_data.append(&mut this_track);
        used_blocks += n_blocks;
    }

    // Create HFE file header
    let header = HFEFileHeader::make(disk_parameters);

    // Construct final HFE file
    let mut hfe_data: Vec<u8> = vec![];
    hfe_data.append(&mut pad_to_block(&header.as_bytes(), HFE_BLOCK_SIZE));
    hfe_data.append(&mut pad_to_block(
        &track_offset_lut.as_bytes(),
        HFE_BLOCK_SIZE * 2,
    ));
    hfe_data.append(&mut track_data);

    Ok(hfe_data)
}

// Pad array of u8 to block size
fn pad_to_block(data: &Vec<u8>, block_size: usize) -> Vec<u8> {
    if data.len() > block_size {
        panic!("HFE file generation: too much data for one block!")
    }

    let data_len = data.len();

    let mut data: Vec<u8> = data.to_vec();

    if data_len != block_size {
        // Add padding
        data.append(&mut vec![0xFF; block_size - data_len % block_size]);
    }

    data
}

// Apply weird HFE format encoding. Example: 1011... -> 10001010...
fn do_weird_hfe_track_encoding(track: &Vec<u8>) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];

    for byte in track.iter() {
        let mut this_byte: u8 = byte.clone();
        let mut byte_encoded: u16 = 0;

        // Shift each bit into number
        for _ in 0..8 {
            // Add 0 bit
            byte_encoded <<= 1;

            // // Add bit to right
            byte_encoded <<= 1;
            byte_encoded |= (this_byte >> 7) as u16;

            // Next bit
            this_byte <<= 1;
        }

        let result = byte_encoded.to_be_bytes().to_vec();

        // Add byte to result
        encoded.append(&mut reverse_bits(&result));
    }

    encoded
}

fn reverse_bits(track: &[u8]) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];

    for byte in track.iter() {
        // Add byte to result
        encoded.push(byte.reverse_bits());
    }

    encoded
}

// Interleave side 1 and side 2 of a track as per HFE spec
fn pack_track(cylinder: &Cylinder) -> (Vec<u8>, u16) {
    let mut data: Vec<u8> = vec![];
    let mut used_blocks: u16 = 0;

    // Divide tracks to parts
    let mut side0_parts = track_to_parts(
        do_weird_hfe_track_encoding(&cylinder[0]),
        (HFE_BLOCK_SIZE / 2) as u16,
    );
    let mut side1_parts = match cylinder.len() == 2 {
        true => track_to_parts(
            do_weird_hfe_track_encoding(&cylinder[1]),
            (HFE_BLOCK_SIZE / 2) as u16,
        ),
        false => vec![],
    };

    // Pack track data
    while (used_blocks as usize) < side0_parts.len() {
        // Get next side 0 part
        data.append(&mut side0_parts[used_blocks as usize]);

        // Get next side 1 part (if present)
        if (used_blocks as usize) < side1_parts.len() {
            // Add side 1 part
            data.append(&mut side1_parts[used_blocks as usize]);
        } else {
            // Add data fill
            let mut fill_part = vec![HFE_PAD_VALUE; (HFE_BLOCK_SIZE / 2) as usize];
            data.append(&mut fill_part);
        }

        // Count number of blocks used by track
        used_blocks += 1;
    }

    (data, used_blocks)
}

// Divide track into 256 byte parts as per HFE spec
fn track_to_parts(track: Vec<u8>, part_size: u16) -> Vec<Vec<u8>> {
    let mut parts: Vec<Vec<u8>> = vec![];

    // Convert track to parts
    let mut bytes_consumed: usize = 0;
    while bytes_consumed < track.len() {
        let remaining_bytes = track.len() - bytes_consumed;

        // Create new part
        if remaining_bytes >= part_size as usize {
            // Valid sector boundary
            parts.push(track[bytes_consumed..bytes_consumed + part_size as usize].to_vec());
        } else {
            // Invalid sector boundary but we ignore errors

            // Fill remaining bytes with zeroes
            let mut this_part: Vec<u8> = vec![0; part_size as usize];
            this_part[0..remaining_bytes].copy_from_slice(&track[bytes_consumed..track.len()]);
            parts.push(this_part);
        }

        bytes_consumed += part_size as usize;
    }

    parts
}

#[derive(Debug, Clone)]
struct HFEFileHeader {
    header_signature: [u8; 8],
    format_revision: u8,
    n_tracks: u8,
    n_sides: u8,
    track_encoding: TrackEncoding,
    bit_rate: u16,
    floppy_rpm: u16,
    floppy_interface_mode: FloppyInterfaceMode,
    dnu: u8,
    track_list_offset: u16,
    write_allowed: u8,
    single_step: u8,
    track0s0_altencoding: u8,
    track0s0_encoding: TrackEncoding,
    track0s1_altencoding: u8,
    track0s1_encoding: TrackEncoding,
}

impl HFEFileHeader {
    pub fn make(disk_parameters: &DiskParameters) -> HFEFileHeader {
        Self {
            header_signature: *b"HXCPICFE",
            format_revision: 0,
            n_tracks: disk_parameters.n_cylinders as u8,
            n_sides: disk_parameters.n_sides as u8,
            track_encoding: floppy_interface_mode_from_track_format(&disk_parameters.track_format),
            bit_rate: disk_parameters.cell_rate,
            floppy_rpm: disk_parameters.rpm,
            floppy_interface_mode: FloppyInterfaceMode::GenericShugartDD,
            dnu: 0x01, // Unused
            track_list_offset: 1,
            write_allowed: 0xFF,        // Not allowed
            single_step: 0xFF,          // Single step
            track0s0_altencoding: 0xFF, // No alt encoding
            track0s0_encoding: TrackEncoding::UnknownEncoding,
            track0s1_altencoding: 0xFF, // No alt encoding
            track0s1_encoding: TrackEncoding::UnknownEncoding,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];

        res.extend_from_slice(&self.header_signature);
        res.extend_from_slice(&self.format_revision.to_le_bytes());
        res.extend_from_slice(&self.n_tracks.to_le_bytes());
        res.extend_from_slice(&self.n_sides.to_le_bytes());
        res.extend_from_slice(&(self.track_encoding as u8).to_le_bytes());
        res.extend_from_slice(&self.bit_rate.to_le_bytes());
        res.extend_from_slice(&self.floppy_rpm.to_le_bytes());
        res.extend_from_slice(&(self.floppy_interface_mode as u8).to_le_bytes());
        res.extend_from_slice(&self.dnu.to_le_bytes());
        res.extend_from_slice(&self.track_list_offset.to_le_bytes());
        res.extend_from_slice(&self.write_allowed.to_le_bytes());
        res.extend_from_slice(&self.single_step.to_le_bytes());
        res.extend_from_slice(&self.track0s0_altencoding.to_le_bytes());
        res.extend_from_slice(&(self.track0s0_encoding as u8).to_le_bytes());
        res.extend_from_slice(&self.track0s1_altencoding.to_le_bytes());
        res.extend_from_slice(&(self.track0s1_encoding as u8).to_le_bytes());

        res
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum TrackEncoding {
    IsoIbmFmEncoding = 0x02,
    UnknownEncoding = 0xFF,
}

fn floppy_interface_mode_from_track_format(track_format: &DiskTrackFormat) -> TrackEncoding {
    match track_format {
        DiskTrackFormat::IBM3470 => TrackEncoding::IsoIbmFmEncoding,
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum FloppyInterfaceMode {
    GenericShugartDD = 0x07,
}

#[derive(Debug, Clone)]
struct HFETrackOffsetLUT {
    entries: Vec<HFETrackOffsetLUTEntry>,
}

impl HFETrackOffsetLUT {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn add_track(&mut self, offset: u16, track_len: u16) {
        self.entries
            .push(HFETrackOffsetLUTEntry { offset, track_len })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];

        for entry in &self.entries {
            data.append(&mut entry.as_bytes());
        }

        data
    }
}

#[derive(Debug, Clone)]
struct HFETrackOffsetLUTEntry {
    offset: u16,
    track_len: u16,
}

impl HFETrackOffsetLUTEntry {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];

        data.extend_from_slice(&self.offset.to_le_bytes());
        data.extend_from_slice(&self.track_len.to_le_bytes());

        data
    }
}
