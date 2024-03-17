use super::{get_new_command_without_sudo, match_rule_without_sudo, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use std::path::Path;

fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn auxiliary_mockable_match_rule<F>(command: &CrabCommand, fn_path_exists: F) -> bool
where
    F: Fn(&str) -> bool,
{
    if let Some(output) = &command.output {
        !command.script_parts.is_empty()
            && fn_path_exists(&command.script_parts[0])
            && output.contains("command not found")
    } else {
        false
    }
}
// Path::new(&command.script_parts[0]).exists()
pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| auxiliary_mockable_match_rule(command, path_exists),
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
    use super::{get_new_command, auxiliary_mockable_match_rule};
    use crate::cli::command::CrabCommand;
    use rstest::rstest;

    #[rstest]
    #[case("main", "main: command not found", true)]
    #[case("main --help", "main: command not found", true)]
    #[case("main", "", false)]
    #[case("main", "main: command not found", false, false)]
    fn test_match(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] is_match: bool,
        #[case] file_exists: bool,
    ) {
        let _file = tempfile::NamedTempFile::new().unwrap();
        if file_exists {
            std::fs::File::create("main").unwrap();
        }
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
        std::fs::remove_file("main").unwrap();
    }

    #[rstest]
    #[case("main --help", vec!["./main --help"])]
    fn test_get_new_command(#[case] command: &str, #[case] expected: Vec<&str>) {
        let mut command = CrabCommand::new(command.to_owned(), Some("".to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
