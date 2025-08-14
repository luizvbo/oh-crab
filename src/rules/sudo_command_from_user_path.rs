use super::{utils::match_rule_with_is_app, Rule};
use crate::utils::replace_argument;
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;
use which::which;

fn is_terminal_command(terminal_command: &str) -> bool {
    which(terminal_command).is_ok()
}

fn get_command_name(command_output: &str) -> Option<String> {
    let re = Regex::new(r"sudo: (.*): command not found").unwrap();
    re.captures(command_output).map(|caps| caps[1].to_owned())
}

fn auxiliary_match_rule<F>(command: &CrabCommand, fn_is_terminal_command: F) -> bool
where
    F: Fn(&str) -> bool,
{
    if let Some(output) = &command.output {
        if output.contains("command not found") {
            if let Some(command_name) = get_command_name(output) {
                fn_is_terminal_command(&command_name)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(
        |command| auxiliary_match_rule(command, is_terminal_command),
        command,
        vec!["sudo"],
        None,
    )
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let command_name = get_command_name(output);
        if let Some(command_name) = command_name {
            vec![replace_argument(
                &command.script,
                &command_name,
                &format!("env \"PATH=$PATH\" {command_name}"),
            )]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "sudo_command_from_user_path".to_owned(),
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
    use super::{auxiliary_match_rule, get_new_command};
    use crate::cli::command::CrabCommand;
    use crate::rules::utils::match_rule_with_is_app;
    use crate::shell::Bash;
    use rstest::rstest;

    fn error_command_not_found(command: &str) -> String {
        format!("sudo: {command}: command not found")
    }

    #[rstest]
    #[case("sudo npm install -g react-native-cli", &error_command_not_found("npm"), true, true)]
    #[case("sudo -u app appcfg update .", &error_command_not_found("appcfg"), true, true)]
    #[case("npm --version", &error_command_not_found("npm"), true, false)]
    #[case("sudo npm --version", "", true, false)]
    #[case("sudo npm --version", &error_command_not_found("npm"), false, false)]
    fn test_match(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] is_terminal_command: bool,
        #[case] is_match: bool,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(
            match_rule_with_is_app(
                |command| auxiliary_match_rule(command, |s| is_terminal_command),
                &command,
                vec!["sudo"],
                None,
            ),
            is_match
        );
    }

    #[rstest]
    #[case("sudo npm install -g react-native-cli", &error_command_not_found("npm"), vec!["sudo env \"PATH=$PATH\" npm install -g react-native-cli"])]
    #[case("sudo -u app appcfg update .", &error_command_not_found("appcfg"), vec!["sudo -u app env \"PATH=$PATH\" appcfg update ."])]
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
