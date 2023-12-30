use crate::{cli::command::CrabCommand, shell::Shell};

use super::Rule;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    command.script.starts_with("apt-get search")
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec!["apt-cache".to_owned() + &command.script[7..]]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "apt_get_search".to_owned(),
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
    use crate::{cli::command::CrabCommand};

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

    parameterized_match_rule_tests! {
        match_rule_1: ("apt-get search foo", ""),
    }

    parameterized_unmatch_rule_tests! {
        unmatch_rule_1: ("ag foo", ""),
        unmatch_rule_2: ("apt-cache search foo", ""),
        unmatch_rule_3: ("aptitude search foo", ""),
        unmatch_rule_4: ("apt search foo", ""),
        unmatch_rule_5: ("apt-get install foo", ""),
        unmatch_rule_6: ("apt-get source foo", ""),
        unmatch_rule_7: ("apt-get clean", ""),
        unmatch_rule_8: ("apt-get remove", ""),
        unmatch_rule_9: ("apt-get update", ""),
    }

    #[test]
    fn test_get_new_command() {
        let mut command = CrabCommand::new("apt-get search foo".to_owned(), Some("".to_owned()), None);
        assert_eq!(get_new_command(&mut command, None)[0], "apt-cache search foo");
    }
}
