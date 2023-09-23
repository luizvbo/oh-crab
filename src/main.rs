extern crate ohcrab; 

use std::env;

use ohcrab::{
    cli::{
        command::run_command,
        parser::{prepare_arguments, get_parser},
    },
    shell::get_bash_type,
    rules::{get_corrected_commands, selected_command}
};

fn main() {
    // Skip the first element of `env::args()` (the name of program)
    let args = env::args().skip(1).collect();
    let args = prepare_arguments(args);
    let mut arg_matches = get_parser().get_matches_from(&args);
    let shell_command = get_bash_type(&arg_matches.remove_one::<String>("shell").unwrap());

    if let Some(command) = arg_matches.remove_many::<String>("command") {
        let crab_command = run_command(command.collect());
        println!("{:?}", crab_command);
        let corrected_commands = get_corrected_commands(&crab_command);
        let selected_command = selected_command(&corrected_commands);
        if let Some(valid_command) = selected_command{
            valid_command.run(crab_command);
        }
    } else {
        let alias_name = arg_matches.get_one::<String>("alias").unwrap();
        println!("{}", shell_command.app_alias(alias_name));
    }
}
