use super::{utils::git::get_command_with_git_support, Rule};

use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        command.script.contains("help") && stdout.contains(" is aliased to ")
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
    if let Some(stdout) = &command.stdout {
        let aliased = stdout.splitn(3, '`').collect::<Vec<_>>();
        if aliased.len() > 2 {
            let aliased = aliased[2].splitn(2, '\'').collect::<Vec<_>>()[0]
                .splitn(2, ' ')
                .collect::<Vec<_>>()[0];
            vec![format!("git help {}", aliased)]
        } else {
            Vec::<String>::new()
        }
    } else {
        Vec::<String>::new()
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_help_aliased".to_owned(),
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

    use rstest::rstest;
    #[rstest]
    #[case("git help st", "`git st' is aliased to `status'", true)]
    #[case("git help ds", "`git ds' is aliased to `diff --staged'", true)]
    #[case("git help status", "GIT-STATUS(1)...Git Manual...GIT-STATUS(1)", false)]
    #[case("git help diff", "GIT-DIFF(1)...Git Manual...GIT-DIFF(1)", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git help st", "`git st' is aliased to `status'", "git help status")]
    #[case(
        "git help ds",
        "`git ds' is aliased to `diff --staged'",
        "git help diff"
    )]
    fn test_get_new_command(#[case] command: &str, #[case] stdout: &str, #[case] expected: &str) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), vec![expected]);
    }
}
