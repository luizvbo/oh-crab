use super::{utils::git::get_command_with_git_support, Rule};
use crate::utils::replace_argument;
use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        command.script.contains("merge")
            && stdout.contains(" - not something we can merge")
            && stdout.contains("Did you mean this?")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    if let Some(stdout) = &command.output {
        let re_unknown_branch = Regex::new(r"merge: (.+) - not something we can merge").unwrap();
        let re_remote_branch = Regex::new(r"Did you mean this\?\n\t([^\n]+)").unwrap();
        if let Some(unknown_branch) = re_unknown_branch.captures(stdout) {
            if let Some(remote_branch) = re_remote_branch.captures(stdout) {
                vec![replace_argument(
                    &command.script,
                    &unknown_branch[1],
                    &remote_branch[1],
                )]
            } else {
                Vec::<String>::new()
            }
        } else {
            Vec::<String>::new()
        }
    } else {
        Vec::<String>::new()
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_merge".to_owned(),
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
