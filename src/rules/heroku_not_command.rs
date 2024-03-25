use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("Run heroku _ to run")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["heroku"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let re = Regex::new(r"Run heroku _ to run ([^.]*)").unwrap();
    let new_cmd = re
        .captures(command.output.as_ref().unwrap())
        .unwrap()
        .get(1)
        .map_or("", |m| m.as_str());
    vec![new_cmd.to_owned()]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "heroku_not_command".to_owned(),
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

    const SUGGEST_OUTPUT: &str = "\n ▸    log is not a heroku command.\n ▸    Perhaps you meant logs?\n ▸    Run heroku _ to run heroku logs.\n ▸    Run heroku help for a list of available commands.";

    #[rstest]
    #[case("heroku log", SUGGEST_OUTPUT, true)]
    #[case("cat log", SUGGEST_OUTPUT, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("heroku log", SUGGEST_OUTPUT, vec!["heroku logs"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
