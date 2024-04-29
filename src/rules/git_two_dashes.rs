use crate::{
    cli::command::CrabCommand,
    rules::{
        utils::git::{get_new_command_with_git_support, match_rule_with_git_support},
        Rule,
    },
    shell::Shell,
    utils::replace_argument,
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        stdout.contains("error: did you mean `") && stdout.contains("` (with two dashes ?)")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    if let Some(stdout) = &command.output {
        if let Some(to) = &stdout.split('`').nth(1) {
            return vec![replace_argument(&command.script, &to[1..], to)];
        }
    }
    return vec![];
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, system_shell)
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
