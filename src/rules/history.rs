use super::Rule;
use crate::cli::command::CrabCommand;
use crate::utils::get_close_matches;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&Box<dyn Shell>>) -> bool {
    command.script == "cargo"
}

pub fn get_rule() -> Rule {
    Rule::new(
        "history".to_owned(),
        None,
        None,
        None,
        match_rule,
        get_new_command,
        None,
    )
}

// def match(command):
//     return len(get_close_matches(command.script,
//                                  get_valid_history_without_current(command)))

// def get_new_command(command):
//     return get_closest(command.script,
//                        get_valid_history_without_current(command))

// priority = 9999
