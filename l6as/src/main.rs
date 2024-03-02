mod args;
mod assembler;
mod file;
mod logging;
mod output;
mod preprocessor;
use std::{path::PathBuf, process::exit};

use assembler::assemble;
use clap::Parser;

use file::FileInclusionCoordinator;
use logging::print_final_error_msg;
use output::{
    write_assembler_binary_output, write_assembler_listing_output, write_preprocessor_output,
};
use preprocessor::preprocess;

const DEFAULT_PREPROCESSOR_OUT_FILE: &str = "a.l6s";
const DEFAULT_ASSEMBLER_BINARY_OUT_FILE: &str = "a.bin";
const DEFAULT_ASSEMBLER_LISTING_OUT_FILE: &str = "a.txt";

fn main() {
    let args = args::Args::parse();

    // Setup file inclusion coordinator
    let mut fi_coord = FileInclusionCoordinator::new();
    fi_coord.add_current_dir().unwrap();
    fi_coord.add_include_dirs(&args.include_dirs).unwrap();

    // Preprocess only?
    let res = if args.preprocess {
        command_preprocessor_only(&args, &mut fi_coord)
    } else {
        command_assemble(&args, &mut fi_coord)
    };

    // Final message
    match res {
        Ok(_) => {
            exit(0);
        }
        Err(_) => {
            exit(1);
        }
    }
}

fn command_preprocessor_only(
    args: &args::Args,
    fi_coord: &mut FileInclusionCoordinator,
) -> Result<(), ()> {
    // Get output file name
    let out_file = match &args.output {
        Some(file) => file.clone(),
        None => PathBuf::from(DEFAULT_PREPROCESSOR_OUT_FILE),
    };

    // Run preprocessor
    match preprocess(&args.input, fi_coord) {
        Ok(lines) => {
            // Write output
            write_preprocessor_output(&out_file, &lines)
        }
        Err(_err) => {
            print_final_error_msg();
            Err(())
        }
    }
}

fn command_assemble(args: &args::Args, fi_coord: &mut FileInclusionCoordinator) -> Result<(), ()> {
    // Has an error happened?
    let mut error_encountered = false;

    // Preprocess
    let code_lines = match preprocess(&args.input, fi_coord) {
        Ok(lines) => lines,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    // Assemble
    let assembled_lines = match assemble(&code_lines) {
        Ok(lines) => lines,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    if !error_encountered {
        if args.listing {
            // Get output file name
            let out_file = match &args.output {
                Some(file) => file.clone(),
                None => PathBuf::from(DEFAULT_ASSEMBLER_BINARY_OUT_FILE),
            };
            // Write binary output
            write_assembler_binary_output(&out_file, &assembled_lines)
        } else {
            // Get output file name
            let out_file = match &args.output {
                Some(file) => file.clone(),
                None => PathBuf::from(DEFAULT_ASSEMBLER_LISTING_OUT_FILE),
            };
            // Write listing
            write_assembler_listing_output(&out_file, &assembled_lines)
        }
    } else {
        logging::print_final_error_msg();
        Err(())
    }
}
