
use super::{ get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule, };
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script_parts.first().map_or(false, |s| s == "pip") && command.script.contains("pip") && output.contains("unknown command") && output.contains("maybe you meant")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo( |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["pip", "pip2", "pip3"], None), command,)
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let broken_cmd = Regex::new(r"ERROR: unknown command \"([^\"]+)\"").unwrap().captures(&command.output).and_then(|caps| caps.get(0).map(|m| m.as_str().to_owned())).unwrap_or("".to_owned());
    let new_cmd = Regex::new(r"maybe you meant \"([^\"]+)\"").unwrap().captures(&command.output).and_then(|caps| caps.get(0).map(|m| m.as_str().to_owned())).unwrap_or("".to_owned());
    vec![command.script.replace(&broken_cmd, &new_cmd)]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "pip_unknown_command".to_owned(),
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

    const ERROR_UNKNOWN_COMMAND: &str = "ERROR: unknown command \"{}\" - maybe you meant \"{}\"";

    #[rstest]
    #[case("pip instatl", format!(ERROR_UNKNOWN_COMMAND, "instatl", "install"), true)]
    #[case("pip i", "ERROR: unknown command \"i\"", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("pip un+install thefuck", "", vec!["pip uninstall thefuck"])]
    #[case("pip instatl", "", vec!["pip install"])]
    fn test_get_new_command( #[case] command: &str, #[case] stdout: &str, #[case] expected: Vec<&str>,) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
