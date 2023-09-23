use crate::{
    cli::command::CrabCommand,
    utils::{get_all_executable, get_close_matches},
};
use similar::DiffableStr;
use which::which;

use super::Rule;

pub fn match_rule(command: &CrabCommand) -> bool {
    which(&command.script_parts[0]).is_err()
        & (if let Some(output) = &command.stderr {
            if output.contains("not found") | output.contains("is not recognized as") {
                true
            } else {
                false
            }
        } else {
            false
        })
        & (get_close_matches(
            &command.script_parts[0],
            get_all_executable()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        )
        .len()
            > 0)
}

pub fn get_new_command(command: &CrabCommand) -> Vec<String> {
    let old_command = &command.script_parts[0];
    // TODO: Check shell history
    let mut new_cmds: Vec<&str> = vec![];
    let executables = get_all_executable();
    let str_executables = executables
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>();
    for cmd in get_close_matches(old_command, &str_executables) {
        if !new_cmds.contains(&cmd) {
            new_cmds.push(cmd);
        }
    }
    new_cmds.iter().map(|s| s.to_string()).collect()
}

pub fn get_rule() -> Rule {
    Rule::new(
        "no_command".to_owned(),
        None,
        None,
        None,
        match_rule,
        get_new_command,
        None,
    )
}
