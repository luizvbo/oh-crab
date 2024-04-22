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
        command.script_parts.contains(&"rebase".to_owned())
            && command.script_parts.contains(&"--continue".to_owned())
            && stdout.contains("No changes - did you forget to use 'git add'?")
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
    vec!["git rebase --skip".to_owned()]
}

pub fn get_new_command(
    command: &mut CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_rebase_no_changes".to_owned(),
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

    const OUTPUT: &str = "Applying: Test commit\nNo changes - did you forget to use 'git add'?\nIf there is nothing left to stage, chances are that something else\nalready introduced the same changes; you might want to skip this patch.\n\nWhen you have resolved this problem, run \"git rebase --continue\".\nIf you prefer to skip this patch, run \"git rebase --skip\" instead.\nTo check out the original branch and stop rebasing, run \"git rebase --abort\".\n";

    #[rstest]
    #[case("git rebase --continue", OUTPUT, true)]
    #[case("git rebase --continue", "", false)]
    #[case("git rebase --skip", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git rebase --continue", OUTPUT, vec!["git rebase --skip"])]
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
