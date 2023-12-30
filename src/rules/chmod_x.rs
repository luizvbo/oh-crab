use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::{get_new_command_without_sudo, match_without_sudo, Rule};

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(stdout) = &command.stdout {
        let metadata = fs::metadata("folder").unwrap();
        let permissions = metadata.permissions();

        command.script.starts_with("./")
            && stdout.to_lowercase().contains("permission denied")
            && Path::new(command.script_parts[0].as_str()).exists()
            && permissions.mode() & 0o100 <= 0
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![system_shell.unwrap().and(vec![
        format!("chmod +x {}", &command.script_parts[0][2..]).as_str(),
        &command.script,
    ])]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "chmod_x".to_owned(),
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
