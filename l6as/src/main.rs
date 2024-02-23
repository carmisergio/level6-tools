mod args;
mod assembler;
mod file;
mod logging;
mod preprocessor;
use std::{path::PathBuf, process::exit};

use assembler::assemble;
use clap::Parser;

use file::{write_file, FileInclusionCoordinator};
use logging::{print_final_error_msg, print_write_file_error_msg};
use preprocessor::{convert_preprocessor_output, preprocess};

const DEFAULT_PREPROCESSOR_OUT_FILE: &str = "a.l6s";

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
            match write_file(&out_file, &convert_preprocessor_output(&lines)) {
                Ok(_) => Ok(()),
                Err(err) => {
                    print_write_file_error_msg(err);
                    Err(())
                }
            }
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
    let _assembled_lines = match assemble(&code_lines) {
        Ok(lines) => lines,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    if !error_encountered {
        // println!("{:#?}", assembled_lines);
        Ok(())
    } else {
        logging::print_final_error_msg();
        Err(())
    }
}
