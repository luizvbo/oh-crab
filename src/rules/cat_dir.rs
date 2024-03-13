use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use std::path::Path;

fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

fn auxiliary_match_rule<F>(command: &CrabCommand, fn_is_dir: F) -> bool
where
    F: Fn(&str) -> bool,
{
    if let Some(output) = &command.output {
        if command.script_parts.len() > 1 {
            output.starts_with("cat: ") && fn_is_dir(&command.script_parts[1])
        } else {
            false
        }
    } else {
        false
    }
}

fn mockable_match_rule<F>(
    command: &mut CrabCommand,
    system_shell: Option<&dyn Shell>,
    fn_is_dir: F,
) -> bool
where
    F: Fn(&str) -> bool,
{
    match_rule_with_is_app(
        |command| auxiliary_match_rule(command, &fn_is_dir),
        command,
        vec!["cat"],
        Some(1),
    )
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    mockable_match_rule(command, system_shell, is_dir)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command.script.replacen("cat", "ls", 1)]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cat_dir".to_owned(),
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
    use super::{get_new_command, mockable_match_rule};
    use crate::cli::command::CrabCommand;
    use rstest::rstest;

    fn is_dir_true(path: &str) -> bool {
        true
    }
    fn is_dir_false(path: &str) -> bool {
        false
    }

    #[rstest]
    #[case("cat foo", "cat: foo: Is a directory\n", true)]
    #[case("cat /foo/bar/", "cat: /foo/bar/: Is a directory\n", true)]
    #[case("cat cat", "cat: cat/: Is a directory\n", true)]
    #[case("cat foo", "foo bar baz", false)]
    #[case("cat foo bar", "foo bar baz", false)]
    #[case("notcat foo bar", "some output", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(
            mockable_match_rule(
                &mut command,
                None,
                match is_match {
                    true => is_dir_true,
                    false => is_dir_false,
                }
            ),
            is_match
        );
    }

    #[rstest]
    #[case("cat foo", "cat: foo: Is a directory\n", vec!["ls foo"])]
    #[case("cat /foo/bar/", "cat: /foo/bar/: Is a directory\n", vec!["ls /foo/bar/"])]
    #[case("cat cat", "cat: cat/: Is a directory\n", vec!["ls cat"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
