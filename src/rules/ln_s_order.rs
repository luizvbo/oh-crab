use super::{get_new_command_without_sudo, match_rule_without_sudo, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use std::path::Path;

fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn get_destination<F>(script_parts: &Vec<String>, fn_file_exists: F) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    for part in script_parts {
        if part != "ln" && part != "-s" && part != "--symbolic" && fn_file_exists(part) {
            return Some(part.to_owned());
        }
    }
    None
}

fn auxiliary_match_rule<F>(command: &CrabCommand, fn_file_exists: F) -> bool
where
    F: Fn(&str) -> bool,
{
    if let Some(output) = &command.output {
        command.script_parts.first().is_some_and(|s| s == "ln")
            && (command.script_parts.contains(&"-s".to_owned())
                || command.script_parts.contains(&"--symbolic".to_owned()))
            && output.contains("File exists")
            && get_destination(&command.script_parts, fn_file_exists).is_some()
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

pub fn auxiliary_get_new_command<F>(command: &CrabCommand, fn_file_exists: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    let destination = get_destination(&command.script_parts, fn_file_exists);
    if let Some(destination) = destination {
        let mut parts = command.script_parts.clone();
        parts.retain(|x| *x != destination);
        parts.push(destination);
        vec![parts.join(" ")]
    } else {
        vec![]
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(
        |command| auxiliary_get_new_command(command, file_exists),
        command,
    )
}

pub fn get_rule() -> Rule {
    Rule::new(
        "ln_s_order".to_owned(),
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
    use super::{auxiliary_get_new_command, auxiliary_match_rule};
    use crate::cli::command::CrabCommand;
    use rstest::rstest;

    const ERROR_FILE_EXISTS: &str = "ln: failed to create symbolic link 'source': File exists";

    #[rstest]
    #[case("ln -s dest source", ERROR_FILE_EXISTS, true, true)]
    #[case("ln dest -s source", ERROR_FILE_EXISTS, true, true)]
    #[case("ln dest source -s", ERROR_FILE_EXISTS, true, true)]
    #[case("ln dest source", ERROR_FILE_EXISTS, true, false)]
    #[case("ls -s dest source", ERROR_FILE_EXISTS, true, false)]
    #[case("ln -s dest source", "", true, false)]
    #[case("ln -s dest source", ERROR_FILE_EXISTS, false, false)]
    fn test_not_match(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] file_exists: bool,
        #[case] is_match: bool,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(auxiliary_match_rule(&command, |path| file_exists), is_match);
    }

    #[rstest]
    #[case("ln -s dest source", ERROR_FILE_EXISTS, vec!["ln -s source dest"])]
    #[case("ln dest -s source", ERROR_FILE_EXISTS, vec!["ln -s source dest"])]
    #[case("ln dest source -s", ERROR_FILE_EXISTS, vec!["ln source -s dest"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(auxiliary_get_new_command(&command, |path| true), expected);
    }
}
