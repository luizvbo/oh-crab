use crate::{cli::command::CrabCommand, shell::Shell};

use super::Rule;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    command.script_parts[0] == "cs"
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec!["cd".to_owned() + &command.script[2..]]
}

/// Redirects cs to cd when there is a typo
/// Due to the proximity of the keys - d and s - this seems like a common typo
///
/// $ cs /etc/
/// cs: command not found
/// $ crab
/// cd /etc/ [enter/↑/↓/ctrl+c]
pub fn get_rule() -> Rule {
    Rule::new(
        "cd_cs".to_owned(),
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

    #[test]
    fn test_match_rule() {
        assert!(match_rule(
            &mut CrabCommand::new(
                "cs".to_owned(),
                Some("cs: command not found".to_owned()),
                None
            ),
            None
        ));
        assert!(match_rule(
            &mut CrabCommand::new(
                "cs /etc/".to_owned(),
                Some("cs: command not found".to_owned()),
                None
            ),
            None
        ));
    }

    #[test]
    fn test_get_new_command() {
        assert_eq!(
            get_new_command(
                &mut CrabCommand::new(
                    "cs /etc/".to_owned(),
                    Some("cs: command not found".to_owned()),
                    None
                ),
                None
            ),
            vec!["cd /etc/"]
        )
    }
}
