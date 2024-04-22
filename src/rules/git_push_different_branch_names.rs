use crate::{
    cli::command::CrabCommand,
    rules::{
        utils::git::{get_new_command_with_git_support, match_rule_with_git_support},
        Rule,
    },
    shell::Shell,
};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        command.script.contains("push")
            && stdout.contains("The upstream branch of your current branch does not match")
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
    if let Some(stdout) = &command.output {
        let re = Regex::new(r"(?m)^ +(git push [^\s]+ [^\s]+)").unwrap();
        let new_command = re
            .captures_iter(stdout)
            .next()
            .map(|cap| cap[1].to_string());
        match new_command {
            Some(new_command) => vec![new_command],
            None => vec![],
        }
    } else {
        vec![]
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
        "git_push_different_branch_names".to_owned(),
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

    const OUTPUT: &str = r#"
fatal: The upstream branch of your current branch does not match
the name of your current branch.  To push to the upstream branch
on the remote, use

    git push origin HEAD:%s

To push to the branch of the same name on the remote, use

    git push origin %s

To choose either option permanently, see push.default in 'git help config'.
"#;

    fn error_msg(localbranch: &str, remotebranch: &str) -> String {
        OUTPUT
            .replace("%s", remotebranch)
            .replace("%s", localbranch)
    }

    #[rstest]
    #[case("git push", &error_msg("foo", "bar"), true)]
    #[case("vim", "", false)]
    #[case("git status", &error_msg("foo", "bar"), false)]
    #[case("git push", "", false)]
    fn test_match(#[case] command: &str, #[case] output: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(output.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git push", &error_msg("foo", "bar"), vec!["git push origin HEAD:bar"])]
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
