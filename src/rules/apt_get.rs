use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;
use which::which;

use super::{get_new_command_without_sudo, match_without_sudo, Rule};

fn get_executable(command: &CrabCommand) -> &str {
    if command.script_parts[0] == "sudo" {
        &command.script_parts[1]
    } else {
        &command.script_parts[0]
    }
}

fn _match_rule(
    command: &mut CrabCommand,
    system_shell: Option<&dyn Shell>,
    mock_which: Option<bool>,
) -> bool {
    if let Some(stdout) = &command.stdout {
        if stdout.contains("not found") || stdout.contains("not installed") {
            if let Some(which_return) = mock_which {
                !which_return
            } else {
                which(get_executable(&command)).is_err()
            }
        } else {
            false
        }
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    _match_rule(command, system_shell, None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let executable = get_executable(command);
    vec![system_shell.unwrap().and(vec![
        &("sudo apt-get install ".to_owned() + executable),
        &command.script,
    ])]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "apt_get".to_owned(),
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
                    let (script, stdout) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!(_match_rule(&mut command, None, Some(false)));
                }
            )*
        }
    }

    macro_rules! parameterized_unmatch_rule_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout, mock_which) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!(!_match_rule(&mut command, None, mock_which));
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
        match_rule_1: ("vim", "vim: command not found"),
        match_rule_2: ("sudo vim", "vim: command not found"),
        match_rule_3: ("vim", "The program \"vim\" is currently not installed. You can install it by typing: sudo apt install vim"),
    }

    parameterized_unmatch_rule_tests! {
        unmatch_rule_1: ("", "", Some(false)),
        unmatch_rule_2: ("vim", "", Some(false)),
        unmatch_rule_4: ("vim", "vim: command not found", Some(true)),
        unmatch_rule_5: ("sudo vim", "vim: command not found", Some(true)),
    }

    parameterized_get_new_command_tests! {
        get_new_command_1: ("vim", "", "sudo apt-get install vim && vim"),
        get_new_command_2: ("git init", "", "sudo apt-get install git && git init"),
        get_new_command_3: ("sudo vim", "", "sudo apt-get install vim && sudo vim"),
        get_new_command_4: ("sudo git init", "", "sudo apt-get install git && sudo git init"),
    }
}
