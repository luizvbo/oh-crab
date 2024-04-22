use crate::{
    cli::command::CrabCommand,
    rules::{match_rule_with_git_support, utils::git::get_new_command_with_git_support},
    Rule,
};
use shell::Shell;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    command.output.as_ref().map_or(false, |o| {
        o.contains("error: did you mean `") && o.contains("` (with two dashes ?)")
    })
}

pub fn match_rule(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    if let Some(captured) = command.output.split('`').nth(1) {
        let corrected_argument = format!("--{}", &captured[1..]);
        let new_script = command
            .script
            .replacen(&captured[1..], &corrected_argument, 1);
        vec![new_script]
    } else {
        vec![command.script.clone()]
    }
}

pub fn get_new_command(
    command: &mut CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_two_dashes".to_owned(),
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

    #[rstest]
    #[case(
        "git add -patch",
        "error: did you mean `--patch` (with two dashes ?)",
        true
    )]
    #[case(
        "git checkout -patch",
        "error: did you mean `--patch` (with two dashes ?)",
        true
    )]
    #[case(
        "git commit -amend",
        "error: did you mean `--amend` (with two dashes ?)",
        true
    )]
    #[case(
        "git push -tags",
        "error: did you mean `--tags` (with two dashes ?)",
        true
    )]
    #[case(
        "git rebase -continue",
        "error: did you mean `--continue` (with two dashes ?)",
        true
    )]
    #[case("git add --patch", "", false)]
    #[case("git checkout --patch", "", false)]
    #[case("git commit --amend", "", false)]
    #[case("git push --tags", "", false)]
    #[case("git rebase --continue", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git add -patch", "error: did you mean `--patch` (with two dashes ?)", vec!["git add --patch"])]
    #[case("git checkout -patch", "error: did you mean `--patch` (with two dashes ?)", vec!["git checkout --patch"])]
    #[case("git commit -amend", "error: did you mean `--amend` (with two dashes ?)", vec!["git commit --amend"])]
    #[case("git push -tags", "error: did you mean `--tags` (with two dashes ?)", vec!["git push --tags"])]
    #[case("git rebase -continue", "error: did you mean `--continue` (with two dashes ?)", vec!["git rebase --continue"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, Some(&system_shell)), expected);
    }
}
