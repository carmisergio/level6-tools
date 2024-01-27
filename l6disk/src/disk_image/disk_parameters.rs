use crate::args;

#[derive(Debug)]
pub enum DiskTrackFormat {
    Level6,
}

#[derive(Debug)]
pub struct DiskParameters {
    pub track_format: DiskTrackFormat,
    pub n_tracks: u16,
    pub sectors_per_track: u16,
    pub bytes_per_sector: u16,
    pub sector_interleave: u16,
}

impl DiskParameters {
    // Construct DiskParameters based on CLI args
    pub fn from_args(args: &args::Args) -> DiskParameters {
        let mut disk_pars: DiskParameters = DiskFormat::LEVEL6;

        // Modify interleave if set
        if let Some(interleave) = args.interleave {
            disk_pars.sector_interleave = interleave;
        }

        disk_pars
    }
}

#[non_exhaustive]
pub struct DiskFormat;

impl DiskFormat {
    // Level6 format
    pub const LEVEL6: DiskParameters = DiskParameters {
        track_format: DiskTrackFormat::Level6,
        n_tracks: 77,
        sectors_per_track: 26,
        bytes_per_sector: 128,
        sector_interleave: 1,
    };
}
