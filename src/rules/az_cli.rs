use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("is not in the") && output.contains("command group")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["az"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let re_mistake = Regex::new(r"(?=az)(?:.*): '(.*)' is not in the '.*' command group.").unwrap();
    let re_options = Regex::new(r"^The most similar choice to '.*' is:\n\\s*(.*)$").unwrap();
    let mistake = re_mistake
        .captures(&command.output.as_ref().unwrap())
        .unwrap()
        .get(1)
        .map_or("", |m| m.as_str());
    let options = re_options
        .captures_iter(&command.output.as_ref().unwrap())
        .map(|cap| cap[1].to_owned())
        .collect::<Vec<_>>();
    options
        .into_iter()
        .map(|o| command.script.replace(mistake, &o))
        .collect()
}

pub fn get_rule() -> Rule {
    Rule::new(
        "az_cli".to_owned(),
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

    const NO_SUGGESTIONS: &str = r#"
az provider: error: the following arguments are required: _subcommand
usage: az provider [-h] {list,show,register,unregister,operation} ...
"#;

    const MISSPELLED_COMMAND: &str = r#"
az: 'providers' is not in the 'az' command group. See 'az --help'.

The most similar choice to 'providers' is:
    provider
"#;

    const MISSPELLED_SUBCOMMAND: &str = r#"
az provider: 'lis' is not in the 'az provider' command group. See 'az provider --help'.

The most similar choice to 'lis' is:
    list
"#;

    #[rstest]
    #[case("az providers", MISSPELLED_COMMAND, true)]
    #[case("az provider lis", MISSPELLED_SUBCOMMAND, true)]
    #[case("az provider", NO_SUGGESTIONS, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("az providers list", MISSPELLED_COMMAND, vec!["az provider list"])]
    #[case("az provider lis", MISSPELLED_SUBCOMMAND, vec!["az provider list"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
