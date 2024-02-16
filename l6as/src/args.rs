use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Input file path
    pub input: PathBuf,

    /// Run preprocessor only
    #[arg(short = 'p', long, action)]
    pub preprocess: bool,

    /// Output file path
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Include directories
    #[arg(short = 'I', long)]
    pub include_dirs: Vec<PathBuf>,
}
