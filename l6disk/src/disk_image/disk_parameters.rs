/******************************************************************************
 * Floppy Format specifications
******************************************************************************/

pub enum DiskEncoding {
    FM,
}

pub enum DiskTrackFormat {
    Level6,
}

pub struct DiskParameters {
    pub sector_size: u16,
    pub sectors_per_track: u16,
    pub encoding: DiskEncoding,
    pub track_format: DiskTrackFormat,
}

#[non_exhaustive]
pub struct DiskFormat;

impl DiskFormat {
    // Level6 format
    pub const LEVEL6: DiskParameters = DiskParameters {
        sector_size: 128,
        sectors_per_track: 26,
        encoding: DiskEncoding::FM,
        track_format: DiskTrackFormat::Level6,
    };
}
