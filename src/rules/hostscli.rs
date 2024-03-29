
use super::{ get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule, };
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

const NO_COMMAND: &str = "Error: No such command";
const NO_WEBSITE: &str = "hostscli.errors.WebsiteImportError";

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script_parts.first().map_or(false, |s| s == "hostscli") && (output.contains(NO_COMMAND) || output.contains(NO_WEBSITE))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo( |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["hostscli"], None), command,)
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    if command.output.contains(NO_WEBSITE) {
        vec!["hostscli websites".to_owned()]
    } else {
        let misspelled_command = Regex::new(r"Error: No such command \".*\"").unwrap().captures(&command.output).and_then(|caps| caps.get(0).map(|m| m.as_str().to_owned())).unwrap_or("".to_owned());
        let commands = vec!["block", "unblock", "websites", "block_all", "unblock_all"];
        vec![command.script.replace(&misspelled_command, &commands.iter().min_by_key(|a| levenshtein::levenshtein(a, &misspelled_command)).unwrap_or(&"".to_string()).to_owned())]
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
    fn test_get_new_command( #[case] command: &str, #[case] stdout: &str, #[case] expected: Vec<&str>,) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
