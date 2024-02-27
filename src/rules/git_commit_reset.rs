use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};

use super::{utils::git::get_command_with_git_support, Rule};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    command.script_parts.contains(&"commit".to_owned())
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    vec!["git reset HEAD~".to_owned()]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_commit_reset".to_owned(),
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

    #[rstest]
    #[case("git commit -m \"test\"", "test output", true)]
    #[case("git commit", "", true)]
    #[case("git branch foo", "", false)]
    #[case("git checkout feature/test_commit", "", false)]
    #[case("git push", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git commit -m \"test commit\"", vec!["git reset HEAD~"])]
    #[case("git commit", vec!["git reset HEAD~"])]
    fn test_get_new_command(#[case] command: &str, #[case] expected: Vec<&str>) {
        let mut command = CrabCommand::new(command.to_owned(), Some("".to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
