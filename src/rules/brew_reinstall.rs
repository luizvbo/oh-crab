use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        let warning_regex = Regex::new(r"Warning: .+ is already installed and up-to-date").unwrap();
        let message_regex = Regex::new(r"To reinstall .+, run `brew reinstall [^`]+`").unwrap();
        command.script.contains("install")
            && warning_regex.is_match(output)
            && message_regex.is_match(output)
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["brew"], Some(2))
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command.script.replace("install", "reinstall")]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "brew_reinstall".to_owned(),
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

    const OUTPUT: &str = "Warning: thefuck 9.9 is already installed and up-to-date\nTo reinstall 9.9, run `brew reinstall thefuck`";

    #[rstest]
    #[case("brew install thefuck", OUTPUT, true)]
    #[case("brew reinstall thefuck", "", false)]
    #[case("brew install foo", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("brew install foo", OUTPUT, vec!["brew reinstall foo"])]
    #[case("brew install bar zap", OUTPUT, vec!["brew reinstall bar zap"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
