use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("Did you mean 'conda")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["conda"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let re = Regex::new(r"'conda ([^']*)'").unwrap();
        let matches = re
            .captures_iter(output)
            .map(|cap| cap[1].to_owned())
            .collect::<Vec<_>>();
        let broken_cmd = matches[0].clone();
        let correct_cmd = matches[1].clone();
        vec![command.script.replace(&broken_cmd, &correct_cmd)]
    } else {
        Vec::<String>::new()
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "conda_mistype".to_owned(),
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

    const MISTYPE_RESPONSE: &str =
        "\n\nCommandNotFoundError: No command 'conda lst'.\nDid you mean 'conda list'?\n\n";

    #[rstest]
    #[case("conda lst", MISTYPE_RESPONSE, true)]
    #[case("codna list", "bash: codna: command not found", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("conda lst", MISTYPE_RESPONSE, vec!["conda list"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
