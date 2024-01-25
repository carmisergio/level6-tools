use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create level6 encoded floppy image
    Create {
        /// Input disk image
        input: PathBuf,

        /// Input disk image
        output: PathBuf,

        /// Ignore image conversion errors
        #[arg(short, long, action)]
        ignore_errors: bool,
    },
}
