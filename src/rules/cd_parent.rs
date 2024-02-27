use crate::{cli::command::CrabCommand, shell::Shell};

use super::Rule;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    command.script == "cd.."
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec!["cd ..".to_owned()]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cd_parent".to_owned(),
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
        match_rule_1: ("cd..", "cd..: command not found"),
    }

    parameterized_unmatch_rule_tests! {
        unmatch_rule_1: ("", ""),
    }

    parameterized_get_new_command_tests! {
        get_new_command_1: ("cd..", "", "cd .."),
    }
}
