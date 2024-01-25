mod args;
mod file;

use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

fn main() {
    let args = args::Args::parse();

    // Decide what command to run
    let res = match args.command {
        args::Command::Create { input, output } => run_create_command(input, output),
    };

    match res {
        Err(msg) => {
            println!("{} {}", "Error: ".red(), msg)
        }
        Ok(()) => {}
    };
}

fn run_create_command(input: PathBuf, output: PathBuf) -> Result<(), String> {
    // Read input file
    let _input_data = match file::read_file(&input) {
        Ok(data) => data,
        Err(err) => return Err(format!("Unable to open \"{}\": {}", input.display(), err)),
    };

    // Convert image
    let generated_data: Vec<u8> = vec![b'a'; 100];

    // Write data to output file
    match file::write_file(&output, generated_data) {
        Ok(()) => {}
        Err(err) => {
            return Err(format!(
                "Unable to write to \"{}\": {}",
                output.display(),
                err
            ))
        }
    };

    Ok(())
}
