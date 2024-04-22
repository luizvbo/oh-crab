use crate::{
    cli::command::CrabCommand,
    rules::{match_rule_with_git_support, utils::git::get_new_command_with_git_support},
    Rule,
};
use shell::Shell;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    command.script.contains("stash")
        && command.script.contains("pop")
        && command.output.as_ref().map_or(false, |o| {
            o.contains("Your local changes to the following files would be overwritten by merge")
        })
}

pub fn match_rule(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    vec!["git add --update && git stash pop && git reset .".to_string()]
}

pub fn get_new_command(
    command: &mut CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_stash_pop".to_owned(),
        Some(900), // priority
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

    const OUTPUT: &str =
        "error: Your local changes to the following files would be overwritten by merge:";

    #[rstest]
    #[case("git stash pop", OUTPUT, true)]
    #[case("git stash", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git stash pop", OUTPUT, vec!["git add --update && git stash pop && git reset ."])]
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
