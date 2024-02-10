mod args;
mod file;
mod logging;
mod preprocessor;

use clap::Parser;

use file::FileInclusionCoordinator;

use crate::preprocessor::parse_source_file;

fn main() {
    let args = args::Args::parse();

    // Setup file inclusion coordinator
    let mut fi_coord = FileInclusionCoordinator::new();
    fi_coord.add_current_dir().unwrap();
    fi_coord.add_include_dirs(&args.include_dirs).unwrap();

    // Has an error happened?
    let mut error = false;

    let source_lines = match parse_source_file(&args.input, &mut fi_coord) {
        Ok(lines) => lines,
        Err(_err) => {
            error = true;
            vec![]
        }
    };

    if !error {
        println!("{:#?}", source_lines);
    } else {
        logging::print_final_error_msg();
    }
}
