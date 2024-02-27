use crate::{cli::command::CrabCommand, shell::Shell};

use super::Rule;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(stdout) = &command.output {
        stdout.contains("Error: This command updates brew itself")
            && stdout.contains("Use `brew upgrade")
            && command.script.contains("update")
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command.script.replace("update", "upgrade")]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "brew_update_formula".to_owned(),
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

    const OUTPUT: &str = "Error: This command updates brew itself, and does not take formula names.\nUse `brew upgrade thefuck`.";

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
                    let (script, formula) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(OUTPUT.to_owned()),
                                None
                            );
                    assert_eq!(get_new_command(&mut command, None)[0], format!("brew upgrade {}", formula));
                }
            )*
        }
    }

    parameterized_match_rule_tests! {
        match_rule_1: ("brew update thefuck", OUTPUT),
    }

    parameterized_unmatch_rule_tests! {
        unmatch_rule_1: ("brew upgrade foo", ""),
        unmatch_rule_2: ("brew update", ""),
    }

    parameterized_get_new_command_tests! {
        get_new_command_1: ("brew update foo", "foo"),
        get_new_command_2: ("brew update bar zap", "bar zap"),
    }
}
