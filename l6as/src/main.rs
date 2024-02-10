mod args;
mod logging;
mod preprocessor;

use clap::Parser;

fn main() {
    let _args = args::Args::parse();

    let lines = preprocessor::parse_source_lines(
        "LDA test, test2\n %include ciaone.l6s\n %endm    helo\n%define ciao=test \n %macro ",
    )
    .unwrap();

    println!("{:#?}", lines);
}
