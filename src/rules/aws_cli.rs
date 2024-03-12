
use crate::{cli::command::CrabCommand, shell::Shell};
use super::{get_new_command_without_sudo, match_rule_with_is_app, Rule};
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

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let re_invalid_choice = Regex::new(r"(?<=Invalid choice: ')(.*)(?=', maybe you meant:)").unwrap();
    let re_options = Regex::new(r"^\\s*\\*\\s(.*)").unwrap();
    let mistake = re_invalid_choice.captures(&command.output.as_ref().unwrap()).unwrap().get(1).map_or("", |m| m.as_str());
    let options = re_options.captures_iter(&command.output.as_ref().unwrap()).map(|cap| cap[1].to_string()).collect::<Vec<_>>();
    options.iter().map(|o| command.script.replace(mistake, o)).collect()
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
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

    #[rstest]
    #[case("aws dynamdb scan", "usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]\nTo see help text, you can run:\n\n  aws help\n  aws <command> help\n  aws <command> <subcommand> help\naws: error: argument command: Invalid choice, valid choices are:\n\ndynamodb                                 | dynamodbstreams\nec2                                      | ecr\n\n\nInvalid choice: 'dynamdb', maybe you meant:\n\n  * dynamodb", true)]
    #[case("aws dynamodb scn", "usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]\nTo see help text, you can run:\n\n  aws help\n  aws <command> help\n  aws <command> <subcommand> help\naws: error: argument operation: Invalid choice, valid choices are:\n\nquery                                    | scan\nupdate-item                              | update-table\n\n\nInvalid choice: 'scn', maybe you meant:\n\n  * scan", true)]
    #[case("aws dynamodb t-item", "usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]\nTo see help text, you can run:\n\n  aws help\n  aws <command> help\n  aws <command> <subcommand> help\naws: error: argument operation: Invalid choice, valid choices are:\n\ndescribe-table                           | get-item\nlist-tables                              | put-item\n\n\nInvalid choice: 't-item', maybe you meant:\n\n  * put-item\n  * get-item", true)]
    #[case("aws dynamodb invalid", "usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]\nTo see help text, you can run:\n\n  aws help\n  aws <command> help\n  aws <command> <subcommand> help\naws: error: argument command: Invalid choice, valid choices are:\n\ndynamodb                                 | dynamodbstreams\nec2                                      | ecr", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("aws dynamdb scan", "usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]\nTo see help text, you can run:\n\n  aws help\n  aws <command> help\n  aws <command> <subcommand> help\naws: error: argument command: Invalid choice, valid choices are:\n\ndynamodb                                 | dynamodbstreams\nec2                                      | ecr\n\n\nInvalid choice: 'dynamdb', maybe you meant:\n\n  * dynamodb", vec!["aws dynamodb scan"])]
    #[case("aws dynamodb scn", "usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]\nTo see help text, you can run:\n\n  aws help\n  aws <command> help\n  aws <command> <subcommand> help\naws: error: argument operation: Invalid choice, valid choices are:\n\nquery                                    | scan\nupdate-item                              | update-table\n\n\nInvalid choice: 'scn', maybe you meant:\n\n  * scan", vec!["aws dynamodb scan"])]
    #[case("aws dynamodb t-item", "usage: aws [options] <command> <subcommand> [<subcommand> ...] [parameters]\nTo see help text, you can run:\n\n  aws help\n  aws <command> help\n  aws <command> <subcommand> help\naws: error: argument operation: Invalid choice, valid choices are:\n\ndescribe-table                           | get-item\nlist-tables                              | put-item\n\n\nInvalid choice: 't-item', maybe you meant:\n\n  * put-item\n  * get-item", vec!["aws dynamodb put-item", "aws dynamodb get-item"])]
    fn test_get_new_command(#[case] command: &str, #[case] stdout: &str, #[case] expected: Vec<&str>) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
