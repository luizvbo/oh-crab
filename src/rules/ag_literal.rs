use crate::{cli::command::CrabCommand, shell::Shell};

use super::Rule;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(stdout) = &command.stdout {
        stdout.ends_with("run ag with -Q\n")
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command.script.replacen("ag", "ag -Q", 1)]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "ag_literal".to_owned(),
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

    const OUTPUT: &str = "ERR: Bad regex! pcre_compile() failed at position 1: missing )\nIf you meant to search for a literal string, run ag with -Q\n";

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
                    let (script, expected) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(OUTPUT.to_owned()),
                                None
                            );
                    assert_eq!(get_new_command(&mut command, None)[0], expected);
                }
            )*
        }
    }

    parameterized_match_rule_tests! {
        match_rule_1: ("ag \\(", OUTPUT),
    }

    parameterized_unmatch_rule_tests! {
        unmatch_rule_1: ("ag foo", ""),
    }

    parameterized_get_new_command_tests! {
        get_new_command_1: ("ag \\(", "ag -Q \\("),
    }
}
