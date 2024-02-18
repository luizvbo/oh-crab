use super::Rule;
use crate::cli::command::CrabCommand;
use crate::shell::Shell;
use crate::utils::{get_close_matches, get_valid_history_without_current};

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    !get_close_matches(
        &command.script,
        get_valid_history_without_current(command, system_shell.unwrap())
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .as_slice(),
        None,
        None,
    )
    .is_empty()
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_close_matches(
        &command.script,
        get_valid_history_without_current(command, system_shell.unwrap())
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .as_slice(),
        None,
        None,
    )
    .iter()
    .map(|&s| s.to_string())
    .collect()
}

pub fn get_rule() -> Rule {
    Rule::new(
        "history".to_owned(),
        None,
        Some(9999),
        None,
        match_rule,
        get_new_command,
        None,
    )
}

// priority = 9999
