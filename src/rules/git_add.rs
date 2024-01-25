use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};
use regex::Regex;
use std::path::Path;

use super::{
    get_new_command_without_sudo, match_without_sudo, utils::git::get_command_with_git_support,
    Rule,
};

fn get_missing_file(command: &CrabCommand, path_exists: Option<bool>) -> Option<String> {
    if let Some(stdout) = &command.stdout {
        let re = Regex::new(r"error: pathspec '([^']*)' did not match any file\(s\) known to git.")
            .unwrap();

        if let Some(captures) = re.captures(stdout) {
            let path = &captures[1];
            if path.is_empty() {
                None
            } else if let Some(path_exists) = path_exists {
                if path_exists {
                    Some(path.to_owned())
                } else {
                    None
                }
            } else if Path::new(&path).exists() {
                Some(path.to_owned())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn mockable_match_rule(command: &CrabCommand, path_exists: Option<bool>) -> bool {
    if let Some(stdout) = &command.stdout {
        stdout.contains("did not match any file(s) known to git.")
            && get_missing_file(command, path_exists).is_some()
    } else {
        false
    }
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    mockable_match_rule(command, None)
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn mockable_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
    path_exists: Option<bool>,
) -> Vec<String> {
    let missing_file = get_missing_file(command, path_exists);
    let str_git_add = format!("git add -- {}", missing_file.unwrap_or("".to_owned()));
    vec![system_shell
        .unwrap()
        .and(vec![&str_git_add, &command.script])]
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    mockable_get_new_command(command, system_shell, None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_add".to_owned(),
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
    use super::{mockable_get_new_command, mockable_match_rule};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;

    macro_rules! parameterized_match_rule_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, target, path_exists, check_eq) = $value;
                    let stdout = format!("error: pathspec '{}' did not match any file(s) known to git.", target);
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert_eq!(mockable_match_rule(&mut command, Some(path_exists)), check_eq);
                }
            )*
        }
    }

    macro_rules! parameterized_get_new_command_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, target, expected) = $value;
                    let stdout = format!("error: pathspec '{}' did not match any file(s) known to git.", target);
                    let system_shell = Bash{};
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert_eq!(mockable_get_new_command(&mut command, Some(&system_shell), Some(true))[0], expected);
                }
            )*
        }
    }

    parameterized_match_rule_tests! {
        match_rule_1: ("git submodule update unknown", "unknown", true, true),
        match_rule_2: ("git commit unknown", "unknown", true, true),
    }

    parameterized_match_rule_tests! {
        unmatch_rule_1: ("git submodule update known", "", true, false),
        unmatch_rule_2: ("git commit known", "", true, false),
    }

    parameterized_get_new_command_tests! {
        get_new_command_1: ("git submodule update unknown", "unknown", "git add -- unknown && git submodule update unknown"),
        get_new_command_2: ("git commit unknown", "unknown", "git add -- unknown && git commit unknown"),
    }
}
