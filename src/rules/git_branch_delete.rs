use crate::{
    cli::command::CrabCommand, rules::utils::{git::match_rule_with_git_support, common::replace_argument}, shell::Shell,
};

use super::{utils::git::get_command_with_git_support, Rule};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        command.script.contains("branch -d")
            && stdout.contains("If you are sure you want to delete it")
    }
    else {
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
    vec![replace_argument(&command.script, "-d", "-D")]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_branch_delete".to_owned(),
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
    #[case("git branch list", true)]
    #[case("", false)]
    #[case("git commit", false)]
    #[case("git branch", false)]
    #[case("git stash list", false)]
    fn test_match(#[case] command: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some("".to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git branch list", vec!["git branch --delete list && git branch"])]
    fn test_get_new_command(#[case] command: &str, #[case] expected: Vec<&str>) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some("".to_owned()), None);
        assert_eq!(get_new_command(&mut command, Some(&system_shell)), expected);
    }
}
