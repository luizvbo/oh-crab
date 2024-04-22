use crate::{
    cli::command::CrabCommand,
    rules::{
        utils::git::{get_new_command_with_git_support, match_rule_with_git_support},
        Rule,
    },
    shell::Shell,
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        command.script.contains("pull")
            && (stdout.contains("You have unstaged changes")
                || stdout.contains("contains uncommitted changes"))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    vec![system_shell
        .unwrap()
        .and(vec!["git stash", "git pull", "git stash pop"])]
}

pub fn get_new_command(
    command: &mut CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_pull_uncommitted_changes".to_owned(),
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
    const OUTPUT: &str = "error: Cannot pull with rebase: You have unstaged changes.";

    #[rstest]
    #[case("git pull", OUTPUT, true)]
    #[case("git pull", "", false)]
    #[case("ls", OUTPUT, false)]
    fn test_match(#[case] command: &str, #[case] output: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(output.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git pull", "error: Cannot pull with rebase: You have unstaged changes.", vec!["git stash && git pull && git stash pop"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] output: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(output.to_owned()), None);
        assert_eq!(
            get_new_command(&mut command, Some(&system_shell)),
            expected.iter().map(|&s| s.to_owned()).collect::<Vec<_>>()
        );
    }
}
