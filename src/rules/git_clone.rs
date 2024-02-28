use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};

use super::{utils::git::get_new_command_with_git_support, Rule};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        stdout.contains("fatal: Too many arguments.") && command.script.contains(" git clone ")
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
    vec![command.script.replacen(" git clone ", " ", 1)]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_clone".to_owned(),
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

    const OUTPUT_CLEAN: &str = r#"
fatal: Too many arguments.

usage: git clone [<options>] [--] <repo> [<dir>]
"#;

    parameterized_match_rule_tests! {
        match_rule,
        match_rule_1: ("git clone git clone foo", OUTPUT_CLEAN, true),
        unmatch_rule_1: ("", "", false),
        unmatch_rule_2: ("git branch", "", false),
        unmatch_rule_3: ("git clone foo", "", false),
        unmatch_rule_4: ("git clone foo bar baz", OUTPUT_CLEAN, false),
    }

    parameterized_get_new_command_tests! {
        get_new_command,
        get_new_command_1: ("git clone git clone foo", OUTPUT_CLEAN, "git clone foo"),
    }
}
