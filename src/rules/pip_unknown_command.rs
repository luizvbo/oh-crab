use super::{match_rule_without_sudo, utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell, utils::replace_argument};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script.contains("pip")
            && output.contains("unknown command")
            && output.contains("maybe you meant")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| {
            match_rule_with_is_app(
                auxiliary_match_rule,
                command,
                vec!["pip", "pip2", "pip3"],
                None,
            )
        },
        command,
    )
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        println!("{output:?}");
        let broken_cmd = Regex::new(r#"ERROR: unknown command "([^"]+)""#)
            .unwrap()
            .captures(output)
            .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
            .unwrap_or_default();
        let new_cmd = Regex::new(r#"maybe you meant "([^"]+)""#)
            .unwrap()
            .captures(output)
            .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
            .unwrap_or_default();
        vec![replace_argument(&command.script, &broken_cmd, &new_cmd)]
    } else {
        vec![]
    }
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
    use rstest::rstest;

    fn pip_unknown_cmd(broken: &str, suggested: &str) -> String {
        format!(r#"ERROR: unknown command "{broken}" - maybe you meant "{suggested}""#)
    }

    #[rstest]
    #[case("pip instatl", &pip_unknown_cmd("instatl", "install"), true)]
    #[case("pip i", r#"ERROR: unknown command "i""#, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("pip un+install thefuck", &pip_unknown_cmd("un+install", "uninstall"), vec!["pip uninstall thefuck"])]
    #[case("pip instatl", &pip_unknown_cmd("instatl", "install"), vec!["pip install"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
