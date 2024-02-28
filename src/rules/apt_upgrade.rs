use crate::{cli::command::CrabCommand, shell::Shell};

use super::{match_rule_without_sudo, Rule};

fn _match_rule(command: &CrabCommand) -> bool {
    (command.script == "apt list --upgradable") & {
        if let Some(stdout) = &command.output {
            stdout.trim().split('\n').count() > 1
        } else {
            false
        }
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(_match_rule, command)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec!["apt upgrade".to_owned()]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "apt_upgrade".to_owned(),
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
    use super::{_match_rule, match_rule};
    use crate::cli::command::CrabCommand;

    #[test]
    fn test_match_rule() {
        assert!(_match_rule(&CrabCommand::new(
            "apt list --upgradable".to_owned(),
            Some("multiple\nlines".to_owned()),
            None
        )));
        assert!(!_match_rule(&CrabCommand::new(
            "sudo apt list --upgradable".to_owned(),
            Some("multiple\nlines".to_owned()),
            None
        )));
        assert!(match_rule(
            &mut CrabCommand::new(
                "sudo apt list --upgradable".to_owned(),
                Some("multiple\nlines".to_owned()),
                None
            ),
            None
        ));
    }
}
