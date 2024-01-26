use crate::{
    cli::command::CrabCommand,
    rules::utils::{common::replace_argument, git::match_rule_with_git_support},
    shell::Shell,
};
use regex::Regex;
use std::path::Path;

use super::{
    get_new_command_without_sudo, match_without_sudo, utils::git::get_command_with_git_support,
    Rule,
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        stdout.contains("Use -f if you really want to add them.")
            && command.script_parts.contains(&"add".to_owned())
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
    vec![replace_argument(&command.script, "add", "add --force")]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_add_force".to_owned(),
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
    use super::{match_rule,get_new_command};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;
    use crate::{parameterized_get_new_command_tests, parameterized_match_rule_tests};

    const OUTPUT: &str = "The following paths are ignored by one of your .gitignore files:\n\
dist/app.js\n\
dist/background.js\n\
dist/options.js\n\
Use -f if you really want to add them.\n";

    parameterized_match_rule_tests! {
        match_rule,
        match_rule_1: ("git add dist/*.js", OUTPUT, true),
        unmatch_rule_1: ("git add dist/*.js", "", false),
    }

    parameterized_get_new_command_tests!{
        get_new_command,
        get_new_command_1: ("git add dist/*.js", OUTPUT, "git add --force dist/*.js"),
    }
}
