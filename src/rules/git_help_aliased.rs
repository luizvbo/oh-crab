use super::{utils::git::get_command_with_git_support, Rule};

use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
    utils::replace_argument,
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    command.script.contains("diff") && !command.script.contains("--staged")
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    vec![replace_argument(&command.script, "diff", "diff --staged")]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_diff_staged".to_owned(),
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
    #[case("git diff foo", "", true)]
    #[case("git diff", "", true)]
    #[case("git diff --staged", "", false)]
    #[case("git tag", "", false)]
    #[case("git branch", "", false)]
    #[case("git log", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git diff", "", "git diff --staged")]
    #[case("git diff foo", "", "git diff --staged foo")]
    fn test_get_new_command(#[case] command: &str, #[case] stdout: &str, #[case] expected: &str) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), vec![expected]);
    }
}
