use super::{get_new_command_without_sudo, match_rule_without_sudo, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use std::path::Path;

fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn auxiliary_match_rule<F>(command: &CrabCommand, fn_file_exists: F) -> bool
where
    F: Fn(&str) -> bool,
{
    if let Some(output) = &command.output {
        output.contains("command not found")
            && command
                .script_parts
                .first()
                .map_or(false, |s| fn_file_exists(s))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| auxiliary_match_rule(command, file_exists),
        command,
    )
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    vec![format!("./{}", command.script)]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "has_exists_script".to_owned(),
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
    use crate::rules::match_rule_without_sudo;
    use rstest::rstest;

    #[rstest]
    #[case("main", "main: command not found", true, true)]
    #[case("main --help", "main: command not found", true, true)]
    #[case("main", "", true, false)]
    fn test_match(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] file_exists: bool,
        #[case] is_match: bool,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(
            match_rule_without_sudo(
                |command| auxiliary_match_rule(command, |s| file_exists),
                &mut command
            ),
            is_match
        );
    }

    #[rstest]
    #[case("main --help", "", vec!["./main --help"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
