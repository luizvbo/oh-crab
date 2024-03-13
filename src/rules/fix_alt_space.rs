use super::{match_rule_without_sudo, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.to_lowercase().contains("command not found") && command.script.contains('\u{00A0}')
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(auxiliary_match_rule, command)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command.script.replace('\u{00A0}', " ")]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "fix_alt_space".to_owned(),
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
    use rstest::rstest;

    #[rstest]
    #[case(
        "ps -ef |\u{00A0}grep foo",
        "-bash: \u{00A0}grep: command not found",
        true
    )]
    #[case("ps -ef | grep foo", "", false)]
    #[case("", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("ps -ef |\u{00A0}grep foo", "", vec!["ps -ef | grep foo"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
