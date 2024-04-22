use crate::{
    cli::command::CrabCommand,
    rules::{match_rule_with_git_support, utils::git::get_new_command_with_git_support},
    Rule,
};
use shell::Shell;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    command
        .output
        .as_ref()
        .map_or(false, |o| o.contains("or stash them"))
}

pub fn match_rule(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    vec![format!("git stash && {}", command.script)]
}

pub fn get_new_command(
    command: &mut CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_stash".to_owned(),
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

    const CHERRY_PICK_ERROR: &str = "\
error: Your local changes would be overwritten by cherry-pick.\n\
hint: Commit your changes or stash them to proceed.\n\
fatal: cherry-pick failed";

    const REBASE_ERROR: &str = "\
Cannot rebase: Your index contains uncommitted changes.\n\
Please commit or stash them.";

    #[rstest]
    #[case("git cherry-pick a1b2c3d", CHERRY_PICK_ERROR, true)]
    #[case("git rebase -i HEAD~7", REBASE_ERROR, true)]
    #[case("git cherry-pick a1b2c3d", "", false)]
    #[case("git rebase -i HEAD~7", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git cherry-pick a1b2c3d", CHERRY_PICK_ERROR, vec!["git stash && git cherry-pick a1b2c3d"])]
    #[case("git rebase -i HEAD~7", REBASE_ERROR, vec!["git stash && git rebase -i HEAD~7"])]
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
