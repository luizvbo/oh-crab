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
        command.script.contains("push")
            && stdout.contains("! [rejected]")
            && stdout.contains("failed to push some refs to")
            && stdout
                .contains("Updates were rejected because the tip of your current branch is behind")
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
    vec![replace_argument(
        &command.script,
        "push",
        "push --force-with-lease",
    )]
}

pub fn get_new_command(
    command: &mut CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_push_force".to_owned(),
        Some(false),
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
To /tmp/foo\n\
 ! [rejected]        master -> master (non-fast-forward)\n\
 error: failed to push some refs to '/tmp/bar'\n\
 hint: Updates were rejected because the tip of your current branch is behind\n\
 hint: its remote counterpart. Integrate the remote changes (e.g.\n\
 hint: 'git pull ...') before pushing again.\n\
 hint: See the 'Note about fast-forwards' in 'git push --help' for details.\n";

    const GIT_UPTODATE: &str = "Everything up-to-date";
    const GIT_OK: &str = "\
Counting objects: 3, done.\n\
Delta compression using up to 4 threads.\n\
Compressing objects: 100% (2/2), done.\n\
Writing objects: 100% (3/3), 282 bytes | 0 bytes/s, done.\n\
Total 3 (delta 0), reused 0 (delta 0)\n\
To /tmp/bar\n\
   514eed3..f269c79  master -> master\n";

    #[rstest]
    #[case("git push", GIT_ERR, true)]
    #[case("git push nvbn", GIT_ERR, true)]
    #[case("git push nvbn master", GIT_ERR, true)]
    #[case("git push", GIT_OK, false)]
    #[case("git push", GIT_UPTODATE, false)]
    #[case("git push nvbn", GIT_OK, false)]
    #[case("git push nvbn master", GIT_UPTODATE, false)]
    fn test_match(#[case] command: &str, #[case] output: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(output.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git push", GIT_ERR, vec!["git push --force-with-lease"])]
    #[case("git push nvbn", GIT_ERR, vec!["git push --force-with-lease nvbn"])]
    #[case("git push nvbn master", GIT_ERR, vec!["git push --force-with-lease nvbn master"])]
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
