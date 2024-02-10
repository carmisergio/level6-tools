use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Input file path
    pub input: PathBuf,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Include directories
    #[arg(short = 'I', long)]
    pub include_dirs: Vec<PathBuf>,
}
