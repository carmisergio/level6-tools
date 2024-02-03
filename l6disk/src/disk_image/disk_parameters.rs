use crate::args;

#[derive(Debug)]
pub enum DiskTrackFormat {
    IBM3470,
}

#[derive(Debug)]
pub struct DiskParameters {
    pub track_format: DiskTrackFormat,
    pub n_sides: u16,
    pub n_cylinders: u16,
    pub sectors_per_track: u16,
    pub bytes_per_sector: u16,
    pub cell_rate: u16, // In kbps
    pub rpm: u16,
    pub sector_interleave: u16,
}

impl DiskParameters {
    // Construct DiskParameters based on CLI args
    pub fn from_args(args: &args::Args) -> DiskParameters {
        // Get default parameters for specified format
        let mut disk_pars = match args.disk_format {
            DiskFormat::LEVEL6 => DiskFormatDefaults::LEVEL6,
            DiskFormat::IBM8DSSD => DiskFormatDefaults::IBM8DSSD,
        };

        // Number of cylinders
        if let Some(cylinders) = args.cylinders {
            disk_pars.n_cylinders = cylinders;
        }

        // Number of heads
        if let Some(heads) = args.heads {
            disk_pars.n_sides = heads;
        }

        // Number of sectors
        if let Some(sectors) = args.sectors {
            disk_pars.sectors_per_track = sectors;
        }

        // Sector size
        if let Some(sector_size) = args.sector_size {
            disk_pars.bytes_per_sector = sector_size;
        }

        // Cell rate
        if let Some(cell_rate) = args.cell_rate {
            disk_pars.cell_rate = cell_rate;
        }

        // Spindle RPM
        if let Some(spindle_rpm) = args.spindle_rpm {
            disk_pars.rpm = spindle_rpm;
        }

        // Interleave
        if let Some(interleave) = args.interleave {
            disk_pars.sector_interleave = interleave;
        }

        disk_pars
    }
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum DiskFormat {
    LEVEL6,
    IBM8DSSD,
}

#[non_exhaustive]
pub struct DiskFormatDefaults;

impl DiskFormatDefaults {
    // Level6 format
    pub const LEVEL6: DiskParameters = DiskParameters {
        track_format: DiskTrackFormat::IBM3470,
        n_sides: 1,
        n_cylinders: 77,
        sectors_per_track: 26,
        bytes_per_sector: 128,
        sector_interleave: 1,
        cell_rate: 500,
        rpm: 360,
    };
    // IBM 8 inch double sided - single density format
    pub const IBM8DSSD: DiskParameters = DiskParameters {
        track_format: DiskTrackFormat::IBM3470,
        n_sides: 2,
        n_cylinders: 77,
        sectors_per_track: 26,
        bytes_per_sector: 128,
        sector_interleave: 1,
        cell_rate: 500,
        rpm: 360,
    };
}
