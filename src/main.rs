//! Command line entry point

pub mod args;
pub mod output;

use clap::Parser;

fn main() {
    let cli = args::Cli::parse();
    let roll_type = cli.roll_type();
    let rolls = rust_roll::roller::roll(roll_type, &mut rand::rng());

    println!("{}", output::format_rolls(&roll_type, &rolls));
}
