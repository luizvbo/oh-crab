use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::{get_new_command_without_sudo, match_without_sudo, Rule};

fn _match_rule(
    command: &mut CrabCommand,
    mock_file_exists: Option<bool>,
    mock_file_access: Option<bool>,
) -> bool {
    if let Some(stdout) = &command.stdout {
        command.script.starts_with("./")
            && stdout.to_lowercase().contains("permission denied")
            && {
                if let Some(file_exists) = mock_file_exists {
                    file_exists
                } else {
                    Path::new(command.script_parts[0].as_str()).exists()
                }
            }
            && {
                if let Some(file_access) = mock_file_access {
                    !file_access
                } else {
                    let metadata = fs::metadata(&command.script_parts[0]).unwrap();
                    let permissions = metadata.permissions();
                    permissions.mode() & 0o100 == 0
                }
            }
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    _match_rule(command, None, None)
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
    use super::{_match_rule, get_new_command};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;

    macro_rules! parameterized_match_rule_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout, file_exists, file_access) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!(_match_rule(&mut command, file_exists, file_access));
                }
            )*
        }
    }

    macro_rules! parameterized_unmatch_rule_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout, file_exists, file_access) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!(!_match_rule(&mut command, file_exists, file_access));
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
                    let system_shell = Bash{};
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert_eq!(get_new_command(&mut command, Some(&system_shell))[0], expected);
                }
            )*
        }
    }

    parameterized_match_rule_tests! {
        match_rule_1: ("./gradlew build", "gradlew: Permission denied", Some(true), Some(false)),
        match_rule_2: ("./install.sh --help", "install.sh: permission denied", Some(true), Some(false)),
    }

    parameterized_unmatch_rule_tests! {
        unmatch_rule_1: ("./gradlew build", "gradlew: Permission denied", Some(true), Some(true)),
        unmatch_rule_2: ("./gradlew build", "gradlew: Permission denied", Some(false), Some(false)),
        unmatch_rule_3: ("./gradlew build", "gradlew: error", Some(true), Some(false)),
        unmatch_rule_4: ("gradlew build", "gradlew: Permission denied", Some(true), Some(false)),
    }

    parameterized_get_new_command_tests! {
        get_new_command_1: ("./gradlew build", "", "chmod +x gradlew && ./gradlew build"),
        get_new_command_2: ("./install.sh --help", "", "chmod +x install.sh && ./install.sh --help"),
    }
}
