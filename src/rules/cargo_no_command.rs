use crate::{cli::command::CrabCommand, shell::Shell};
use super::{get_new_command_without_sudo, match_rule_with_is_app, Rule};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script_parts.get(1).is_some() && output.to_lowercase().contains("no such subcommand") && output.contains("Did you mean")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["cargo"], Some(1))
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let re = Regex::new(r"Did you mean `([^`]*)`").unwrap();
    let broken = command.script_parts[1].clone();
    let fix = re.captures(&command.output.as_ref().unwrap()).unwrap().get(1).map_or("", |m| m.as_str());
    vec![command.script.replace(&broken, fix)]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cargo_no_command".to_owned(),
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

    const NO_SUCH_SUBCOMMAND_OLD: &str = "No such subcommand\n\n        Did you mean `build`?\n";
    const NO_SUCH_SUBCOMMAND: &str = "error: no such subcommand\n\n\tDid you mean `build`?\n";

    #[rstest]
    #[case("cargo buid", NO_SUCH_SUBCOMMAND_OLD, true)]
    #[case("cargo buils", NO_SUCH_SUBCOMMAND, true)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("cargo buid", NO_SUCH_SUBCOMMAND_OLD, vec!["cargo build"])]
    #[case("cargo buils", NO_SUCH_SUBCOMMAND, vec!["cargo build"])]
    fn test_get_new_command(#[case] command: &str, #[case] stdout: &str, #[case] expected: Vec<&str>) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
