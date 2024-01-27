use clap::Parser;
use std::path::PathBuf;

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

    /// Disk sector interleave
    #[arg(short = 'i', long, default_value = None, value_parser=clap::value_parser!(u16).range(1..))]
    pub interleave: Option<u16>,
}
