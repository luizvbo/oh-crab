use super::{Rule, utils::match_rule_with_is_app};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("usage:") && output.contains("maybe you meant:")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["aws"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let re_invalid_choice =
        Regex::new(r"(?<=Invalid choice: ')(.*)(?=', maybe you meant:)").unwrap();
    let re_options = Regex::new(r"^\\s*\\*\\s(.*)").unwrap();
    let mistake = re_invalid_choice
        .captures(&command.output.as_ref().unwrap())
        .unwrap()
        .get(1)
        .map_or("", |m| m.as_str());
    let options = re_options
        .captures_iter(&command.output.as_ref().unwrap())
        .map(|cap| cap[1].to_string())
        .collect::<Vec<_>>();
    options
        .iter()
        .map(|o| command.script.replace(mistake, o))
        .collect()
}

pub fn get_rule() -> Rule {
    Rule::new(
        "aws_cli".to_owned(),
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
usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]
To see help text, you can run:

  aws help
  aws <command> help
  aws <command> <subcommand> help
aws: error: argument command: Invalid choice, valid choices are:

dynamodb                                 | dynamodbstreams
ec2                                      | ecr
"#;

    const MISSPELLED_COMMAND: &str = r#"
usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]
To see help text, you can run:

  aws help
  aws <command> help
  aws <command> <subcommand> help
aws: error: argument command: Invalid choice, valid choices are:

dynamodb                                 | dynamodbstreams
ec2                                      | ecr


Invalid choice: 'dynamdb', maybe you meant:

  * dynamodb
"#;

    const MISSPELLED_SUBCOMMAND: &str = r#"
usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]
To see help text, you can run:

  aws help
  aws <command> help
  aws <command> <subcommand> help
aws: error: argument operation: Invalid choice, valid choices are:

query                                    | scan
update-item                              | update-table


Invalid choice: 'scn', maybe you meant:

  * scan
"#;

    const MISSPELLED_SUBCOMMAND_WITH_MULTIPLE_OPTIONS: &str = r#"
usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]
To see help text, you can run:

  aws help
  aws <command> help
  aws <command> <subcommand> help
aws: error: argument operation: Invalid choice, valid choices are:

describe-table                           | get-item
list-tables                              | put-item


Invalid choice: 't-item', maybe you meant:

  * put-item
  * get-item
"#;

    #[rstest]
    #[case("aws dynamdb scan", MISSPELLED_COMMAND, true)]
    #[case("aws dynamodb scn", MISSPELLED_SUBCOMMAND, true)]
    #[case("aws dynamodb t-item", MISSPELLED_SUBCOMMAND_WITH_MULTIPLE_OPTIONS, true)]
    #[case("aws dynamodb invalid", NO_SUGGESTIONS, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("aws dynamdb scan", MISSPELLED_COMMAND, vec!["aws dynamodb scan"])]
    #[case("aws dynamodb scn", MISSPELLED_SUBCOMMAND, vec!["aws dynamodb scan"])]
    #[case("aws dynamodb t-item", MISSPELLED_SUBCOMMAND_WITH_MULTIPLE_OPTIONS, vec!["aws dynamodb put-item", "aws dynamodb get-item"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
