use super::{
    get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule,
};
use crate::{cli::command::CrabCommand, shell::Shell, utils::replace_command};
use regex::Regex;

const NO_COMMAND: &str = "Error: No such command";
const NO_WEBSITE: &str = "hostscli.errors.WebsiteImportError";

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains(NO_COMMAND) || output.contains(NO_WEBSITE)
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["hostscli"], None),
        command,
    )
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    if let Some(output) = &command.output {
        if output.contains(NO_WEBSITE) {
            vec!["hostscli websites".to_owned()]
        } else {
            let misspelled_command = Regex::new(r#"Error: No such command "(.*)""#)
                .unwrap()
                .captures(output)
                .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()));
            match misspelled_command {
                None => vec![],
                Some(misspelled_command) => replace_command(
                    command,
                    &misspelled_command,
                    vec!["block", "unblock", "websites", "block_all", "unblock_all"],
                ),
            }
        }
    } else {
        vec![]
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "hostscli".to_owned(),
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
    use crate::shell::Bash;
    use rstest::rstest;

    const ERROR_NO_WEBSITE: &str = "\nhostscli.errors.WebsiteImportError:\n\nNo Domain list found for website: a_website_that_does_not_exist\n\nPlease raise a Issue here: https://github.com/dhilipsiva/hostscli/issues/new\nif you think we should add domains for this website.\n\ntype `hostscli websites` to see a list of websites that you can block/unblock\n";

    #[rstest]
    #[case("hostscli block a_website_that_does_not_exist", ERROR_NO_WEBSITE, true)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("hostscli block a_website_that_does_not_exist", ERROR_NO_WEBSITE, vec!["hostscli websites"])]
    #[case("hostscli websitess", r#"Error: No such command "websitess""#, vec!["hostscli websites"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
