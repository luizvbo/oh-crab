extern crate ohcrab;

use std::{
    env,
    process::{Command, Output, Stdio},
};

const ARGUMENT_PLACEHOLDER: &str = "OHCRAB_ARGUMENT_PLACEHOLDER";

fn main() {
    ohcrab::handler()
    // // Skip the first element of `env::args()` (the name of program)
    // let args = env::args().skip(1).collect();
    // let args = prepare_arguments(args);
    // let arg_matches = get_argument_parser().get_matches_from(&args);

    // read_arguments(arg_matches);
    // println!("{:?}", args)
}
