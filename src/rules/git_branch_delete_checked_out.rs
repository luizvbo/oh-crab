use super::{utils::git::get_command_with_git_support, Rule};
use crate::utils::replace_argument;
use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        (command.script.contains("branch -d") || command.script.contains("branch -D"))
            && stdout.contains("error: Cannot delete branch '")
            && stdout.contains("' checked out at '")
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
    vec![system_shell.unwrap().and(vec![
        "git checkout master",
        &replace_argument(&command.script, "-d", "-D"),
    ])]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_branch_delete_checked_out".to_owned(),
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

    const OUTPUT: &str = "error: Cannot delete branch 'foo' checked out at '/bar/foo'";

    use rstest::rstest;

    #[rstest]
    #[case("git branch -d foo", OUTPUT, true)]
    #[case("git branch -D foo", OUTPUT, true)]
    #[case("git branch -d foo", "Deleted branch foo (was a1b2c3d).", false)]
    #[case("git branch -D foo", "Deleted branch foo (was a1b2c3d).", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git branch -d foo", OUTPUT, vec!["git checkout master && git branch -D foo"])]
    #[case("git branch -D foo", OUTPUT, vec!["git checkout master && git branch -D foo"])]
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
