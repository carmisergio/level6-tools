mod args;
mod preprocessor;

use clap::Parser;

fn main() {
    let _args = args::Args::parse();

    let lines = preprocessor::parse_source_lines(
        "%define TEST=10 big computers\n LDR $R1, 0x0FA0 \n     %define BEST_MINICOMPUTER = Honeywell Level 6 \n %include extra.l6s \n %macro DOTHING(arg1, arg2) \n %endm         \n",
    )
    .unwrap();

    println!("{:#?}", lines);
}
