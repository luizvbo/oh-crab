use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

use super::{get_new_command_without_sudo, match_without_sudo, Rule};

fn _match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        let stdout = stdout.to_lowercase();
        command.script.starts_with("cd ")
            && (stdout.contains("no such file or directory")
                || stdout.contains("cd: can't cd to")
                || stdout.contains("does not exist"))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_without_sudo(_match_rule, command)
}

pub fn get_new_command_mkdir(command: &CrabCommand) -> Vec<String> {
    let re = Regex::new(r"^cd (.*)").unwrap();
    let repl = |caps: &regex::Captures| format!("mkdir -p {} && cd {}", &caps[1], &caps[1]);
    vec![re.replace(&command.script, repl).to_string()]
}

fn _get_new_command(command: &CrabCommand) -> Vec<String> {
    vec!["apt list --upgradable".to_owned()]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(get_new_command_mkdir, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cd_mkdir".to_owned(),
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

    macro_rules! parameterized_match_rule_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!(match_rule(&mut command, None));
                }
            )*
        }
    }

    macro_rules! parameterized_unmatch_rule_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!(!match_rule(&mut command, None));
                }
            )*
        }
    }

    macro_rules! parameterized_get_new_command_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout, expected) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert_eq!(get_new_command(&mut command, None)[0], expected);
                }
            )*
        }
    }

    parameterized_match_rule_tests! {
        match_rule_1: ("cd foo", "cd: foo: No such file or directory"),
        match_rule_2: ("cd foo/bar/baz", "cd: foo: No such file or directory"),
        match_rule_3: ("cd foo/bar/baz", "cd: can't cd to foo/bar/baz"),
        match_rule_4: ("cd /foo/bar/", "cd: The directory \"/foo/bar/\" does not exist"),
    }

    parameterized_unmatch_rule_tests! {
        unmatch_rule_1: ("cd foo", ""),
        unmatch_rule_2: ("", ""),
    }

    parameterized_get_new_command_tests! {
        get_new_command_1: ("cd foo", "", "mkdir -p foo && cd foo"),
        get_new_command_2: ("cd foo/bar/baz", "", "mkdir -p foo/bar/baz && cd foo/bar/baz"),
    }
}
