mod args;
mod disk_image;
mod file;

use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

fn main() {
    let args = args::Args::parse();

    // Decide what command to run
    let res = match args.command {
        args::Command::Create {
            input,
            output,
            ignore_errors,
        } => run_create_command(input, output, ignore_errors),
    };

    match res {
        Err(msg) => {
            println!("{} {}", "Error: ".red(), msg)
        }
        Ok(()) => {}
    };
}

fn run_create_command(input: PathBuf, output: PathBuf, ignore_errors: bool) -> Result<(), String> {
    // Read input file
    let input_data = match file::read_file(&input) {
        Ok(data) => data,
        Err(err) => return Err(format!("Unable to open \"{}\": {}", input.display(), err)),
    };

    // Convert image
    let generated_data = match disk_image::encode_disk_image(
        input_data,
        disk_image::EncodeOpts {
            ignore_errors,
            format: disk_image::DiskFormat::LEVEL6,
        },
    ) {
        Ok(data) => data,
        Err(err) => return Err(format!("Image conversion error: {}", err)),
    };

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
