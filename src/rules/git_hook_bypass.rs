
use crate::{
    cli::command::CrabCommand,
    rules::{utils::git::get_new_command_with_git_support, match_rule_with_git_support},
    Rule,
};
use shell::Shell;

static HOOKED_COMMANDS: &[&str] = &["am", "commit", "push"];

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    HOOKED_COMMANDS.iter().any(|&hooked_command| command.script.contains(hooked_command))
}

pub fn match_rule(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(command: &CrabCommand, _system_shell: Option<&dyn Shell>) -> Vec<String> {
    HOOKED_COMMANDS.iter().filter_map(|&hooked_command| {
        if command.script.contains(hooked_command) {
            Some(command.script.replacen(hooked_command, &format!("{} --no-verify", hooked_command), 1))
        } else {
            None
        }
    }).collect()
}

pub fn get_new_command(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_hook_bypass".to_owned(),
        Some(1100),
        Some(false),
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
    #[case("git am", "", true)]
    #[case("git commit", "", true)]
    #[case("git commit -m 'foo bar'", "", true)]
    #[case("git push", "", true)]
    #[case("git push -u foo bar", "", true)]
    #[case("git add foo", "", false)]
    #[case("git status", "", false)]
    #[case("git diff foo bar", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git am", "", vec!["git am --no-verify"])]
    #[case("git commit", "", vec!["git commit --no-verify"])]
    #[case("git commit -m 'foo bar'", "", vec!["git commit --no-verify -m 'foo bar'"])]
    #[case("git push", "", vec!["git push --no-verify"])]
    #[case("git push -p", "", vec!["git push --no-verify -p"])]
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
