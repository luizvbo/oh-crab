use crate::{cli::command::CrabCommand, shell::Shell};

use super::Rule;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    command.script == "cargo"
}

pub fn get_new_command(command: &CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec!["cargo build".to_owned()]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cargo".to_owned(),
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
    use crate::{cli::command::CrabCommand, rules::cargo::match_rule, shell::Zsh};

    #[test]
    fn test_match_rule() {
        assert!(match_rule(
            &mut CrabCommand::new("cargo".to_owned(), Some("multiple\nlines".to_owned()), None),
            None
        ));
        assert!(!match_rule(
            &mut CrabCommand::new(
                "acargo".to_owned(),
                Some("multiple\nlines".to_owned()),
                None
            ),
            None
        ));
    }
}
