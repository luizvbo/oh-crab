#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::type_complexity)]

pub mod cli;
pub mod rules;
pub mod shell;
mod ui;
mod utils;

use cli::{
    command::run_command,
    parser::{get_parser, prepare_arguments},
};
use rules::{get_corrected_commands, selected_command};
use shell::get_bash_type;
use std::env;

const ARGUMENT_PLACEHOLDER: &str = "OHCRAB_ARGUMENT_PLACEHOLDER";
const ENV_VAR_NAME_HISTORY: &str = "OHCRAB_COMMAND_HISTORY";
const ENV_VAR_NAME_ALIAS: &str = "OHCRAB_ALIAS";
const ENV_VAR_NAME_SHELL: &str = "OHCRAB_SHELL";

fn main() {
    env_logger::init();
    // Skip the first element of `env::args()` (the name of program)
    let args = env::args().skip(1).collect();
    let args = prepare_arguments(args);
    let mut arg_matches = get_parser().get_matches_from(args);
    let system_shell = get_bash_type(&arg_matches.remove_one::<String>("shell").unwrap());

    if let Some(command) = arg_matches.remove_many::<String>("command") {
        let command_vec = command.collect();
        log::debug!("Retrieved command(s): {:?}", command_vec);
        let mut crab_command = run_command(command_vec, &*system_shell);
        log::debug!("Crab command: {:?}", crab_command);
        let corrected_commands = get_corrected_commands(&mut crab_command, &*system_shell);
        log::debug!(
            "Candidate command(s): {:?}",
            corrected_commands
                .iter()
                .map(|cmd| cmd.script.to_owned())
                .collect::<Vec<_>>()
        );
        let selected_command = selected_command(&corrected_commands);
        // Print a new line after the menu
        eprintln!();
        if let Some(valid_command) = selected_command {
            log::debug!("Command selected: {:?}", valid_command);
            valid_command.run(crab_command);
        }
    } else {
        let alias_name = arg_matches.get_one::<String>("alias").unwrap();
        println!("{}", system_shell.app_alias(alias_name));
    }
}
