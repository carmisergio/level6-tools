use clap::Parser;
use std::path::PathBuf;

use crate::disk_image::disk_parameters::DiskFormat;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Input data disk image
    pub input: PathBuf,

    /// Output raw disk image
    pub output: PathBuf,

    /// Ignore image conversion errors
    #[arg(short = 'p', long, action)]
    pub ignore_errors: bool,

    /// Disk format
    #[arg(value_enum, short = 'd', long, default_value_t = DiskFormat::LEVEL6)]
    pub disk_format: DiskFormat,

    /// Number of cylinders
    #[arg(short = 'c', long, default_value = None, value_parser=clap::value_parser!(u16).range(0..))]
    pub cylinders: Option<u16>,

    /// Number of heads (sides)
    #[arg(short = 'e', long, default_value = None, value_parser=clap::value_parser!(u16).range(1..))]
    pub heads: Option<u16>,

    /// Number of Sectors per track
    #[arg(short = 's', long, default_value = None, value_parser=clap::value_parser!(u16).range(0..))]
    pub sectors: Option<u16>,

    /// Sector size
    #[arg(short = 'b', long, default_value = None, value_parser=clap::value_parser!(u16).range(1..))]
    pub sector_size: Option<u16>,

    /// Disk sector interleave
    #[arg(short = 'i', long, default_value = None, value_parser=clap::value_parser!(u16).range(1..))]
    pub interleave: Option<u16>,
}
