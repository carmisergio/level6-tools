mod args;
mod file;
mod logging;
mod preprocessor;

use clap::Parser;

use file::FileInclusionCoordinator;
use preprocessor::preprocess;

fn main() {
    let args = args::Args::parse();

    // Setup file inclusion coordinator
    let mut fi_coord = FileInclusionCoordinator::new();
    fi_coord.add_current_dir().unwrap();
    fi_coord.add_include_dirs(&args.include_dirs).unwrap();

    // Has an error happened?
    let mut error = false;

    let code = match preprocess(&args.input, &mut fi_coord) {
        Ok(lines) => lines,
        Err(_err) => {
            error = true;
            vec![]
        }
    };

    if !error {
        println!("{:#?}", code);
    } else {
        logging::print_final_error_msg();
    }
}
