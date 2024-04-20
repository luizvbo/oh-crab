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
    if let Some(output) = &command.output {
        output.contains("fatal: Not a git repository")
            && output.contains(
                "Stopping at filesystem boundary (GIT_DISCOVERY_ACROSS_FILESYSTEM not set).",
            )
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    vec![replace_argument(&command.script, "pull", "clone")]
}

pub fn get_new_command(
    command: &mut CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_pull_clone".to_owned(),
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

    const GIT_ERR: &str = "\
fatal: Not a git repository (or any parent up to mount point /home)\n\
Stopping at filesystem boundary (GIT_DISCOVERY_ACROSS_FILESYSTEM not set).\n";

    #[rstest]
    #[case("git pull git@github.com:mcarton/thefuck.git", GIT_ERR, true)]
    fn test_match(#[case] command: &str, #[case] output: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(output.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git pull git@github.com:mcarton/thefuck.git", GIT_ERR, vec!["git clone git@github.com:mcarton/thefuck.git"])]
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
