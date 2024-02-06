use std::ops::Index;

use super::{utils::git::get_command_with_git_support, Rule};
use crate::{
    cli::command::CrabCommand,
    rules::utils::{common::replace_argument, git::match_rule_with_git_support},
    shell::Shell,
};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        command.script_parts.contains(&"push".to_owned())
            && stdout.contains("git push --set-upstream")
    } else {
        false
    }
}

fn get_upstream_option_index(command_parts: &Vec<&str>) -> Option<usize> {
    if command_parts.contains(&"--set-upstream") {
        command_parts.iter().position(|&r| r == "--set-upstream")
    } else if command_parts.contains(&"-u") {
        command_parts.iter().position(|&r| r == "-u")
    } else {
        None
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    // If --set-upstream or -u are passed, remove it and its argument. This is
    // because the remaining arguments are concatenated onto the command suggested
    // by git, which includes --set-upstream and its argument
    let mut command_parts = command.script_parts.clone();
    let upstream_option_index = get_upstream_option_index(&command_parts);

    if let Some(index) = upstream_option_index {
        command_parts.remove(index);
        if command_parts.len() > index {
            command_parts.remove(index);
        }
    } else {
        let push_idx = command_parts.iter().position(|&r| r == "push").unwrap() + 1;
        while command_parts.len() > push_idx && command_parts[command_parts.len() - 1].chars().next().unwrap() != '-' {
            command_parts.pop();
        }
    }

    let re = Regex::new(r"git push (.*)").unwrap();
    let arguments = re.captures(&command.output).unwrap()[1].replace("'", r"\'").trim().to_string();
    replace_argument(&command_parts.join(" "), "push", &format!("push {}", arguments))

}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_push".to_owned(),
        None,
        None,
        None,
        match_rule,
        get_new_command,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::{get_new_command, match_rule};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;
    use crate::{parameterized_get_new_command_tests, parameterized_match_rule_tests};

    const OUTPUT: &str =
        "merge: local - not something we can merge\n\nDid you mean this?\n\tremote/local";

    parameterized_match_rule_tests! {
        match_rule,
        match_rule_01: ("git merge test", OUTPUT, true),
        unmatch_rule_01: ("git merge master", "", false),
        unmatch_rule_02: ("ls", OUTPUT, false),
    }

    parameterized_get_new_command_tests! {
        get_new_command,
        get_new_command_1: ("git merge local", OUTPUT, "git merge remote/local"),
        get_new_command_2: ("git merge -m \"test\" local", OUTPUT, "git merge -m \"test\" remote/local"),
        get_new_command_3: ("git merge -m \"test local\" local", OUTPUT, "git merge -m \"test local\" remote/local"),
    }
}
