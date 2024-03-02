use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Input file path
    pub input: PathBuf,

    /// Run preprocessor only
    #[arg(short = 'p', long, action, conflicts_with = "listing")]
    pub preprocess: bool,

    /// Produce listing
    #[arg(short = 'l', long, action, conflicts_with = "preprocess")]
    pub listing: bool,

    /// Output file path
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Include directories
    #[arg(short = 'I', long)]
    pub include_dirs: Vec<PathBuf>,
}
