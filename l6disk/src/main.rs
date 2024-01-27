mod args;
mod disk_image;
mod file;

use clap::Parser;
use colored::Colorize;
use disk_image::{disk_parameters, DiskFormat, DiskParameters};

fn main() {
    let args = args::Args::parse();

    // Decide what command to run
    let res = run_create_command(args);

    match res {
        Err(msg) => {
            println!("{} {}", "error: ".bright_red().bold(), msg)
        }
        Ok(()) => {}
    };
}

fn run_create_command(args: args::Args) -> Result<(), String> {
    // Read input file
    let input_data = match file::read_file(&args.input) {
        Ok(data) => data,
        Err(err) => {
            return Err(format!(
                "Unable to open \"{}\": {}",
                args.input.display(),
                err
            ))
        }
    };

    // Convert image
    let generated_data = match disk_image::encode_disk_image(
        input_data,
        disk_image::EncodeOpts {
            ignore_errors: args.ignore_errors,
            disk_parameters: DiskParameters::from_args(&args),
            out_file_format: disk_image::RawImageFormat::HFE,
        },
    ) {
        Ok(data) => data,
        Err(err) => return Err(format!("Image conversion error: {}", err)),
    };

    // Write data to output file
    match file::write_file(&args.output, generated_data) {
        Ok(()) => {}
        Err(err) => {
            return Err(format!(
                "Unable to write to \"{}\": {}",
                args.output.display(),
                err
            ))
        }
    };

    Ok(())
}
