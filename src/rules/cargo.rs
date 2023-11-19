use crate::cli::command::CrabCommand;

use super::Rule;

pub fn match_rule(command: &mut CrabCommand) -> bool {
    command.script == "cargo"
}

pub fn get_new_command(command: &CrabCommand) -> Vec<String> {
    vec!["cargo build".to_owned()]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cargo".to_owned(),
        None,
        None,
        None,
        match_rule,
        get_new_command,
        None,
    )
}
