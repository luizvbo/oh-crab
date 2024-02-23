use super::{utils::git::get_command_with_git_support, Rule};
use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};

fn first_0flag(script_parts: &Vec<String>) -> Option<&String> {
    script_parts
        .iter()
        .find(|p| p.len() == 2 && p.starts_with("0"))
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    command.script_parts.get(1) == Some(&"branch".to_owned())
        && first_0flag(&command.script_parts).is_some()
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    if let Some(branch_name) = first_0flag(&command.script_parts) {
        let fixed_flag = branch_name.replace("0", "-");
        let fixed_script = command.script.replace(branch_name, &fixed_flag);
        if let Some(stdout) = &command.stdout {
            if stdout.contains("A branch named '") && stdout.contains("' already exists.") {
                let delete_branch = format!("git branch -D {}", branch_name);
                vec![delete_branch, fixed_script]
            } else {
                vec![fixed_script]
            }
        } else {
            vec![fixed_script]
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
        "git_branch_0flag".to_owned(),
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

    const OUTPUT_BRANCH_EXISTS: &str = "fatal: A branch named 'bar' already exists.";
    const OUTPUT_NOT_VALID_OBJECT: &str = "fatal: Not a valid object name: 'bar'.";

    use rstest::rstest;

    #[rstest]
    #[case("git branch 0a", OUTPUT_BRANCH_EXISTS, true)]
    #[case("git branch 0d", OUTPUT_BRANCH_EXISTS, true)]
    #[case("git branch 0f", OUTPUT_BRANCH_EXISTS, true)]
    #[case("git branch 0r", OUTPUT_BRANCH_EXISTS, true)]
    #[case("git branch 0v", OUTPUT_BRANCH_EXISTS, true)]
    #[case("git branch 0d foo", OUTPUT_BRANCH_EXISTS, true)]
    #[case("git branch 0D foo", OUTPUT_BRANCH_EXISTS, true)]
    #[case("git branch -a", "", false)]
    #[case("git branch -r", "", false)]
    #[case("git branch -v", "", false)]
    #[case("git branch -d foo", "", false)]
    #[case("git branch -D foo", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git branch 0a", OUTPUT_BRANCH_EXISTS, vec!["git branch -D 0a", "git branch -a"])]
    #[case("git branch 0v", OUTPUT_BRANCH_EXISTS, vec!["git branch -D 0v", "git branch -v"])]
    #[case("git branch 0d foo", OUTPUT_BRANCH_EXISTS, vec!["git branch -D 0d", "git branch -d foo"])]
    #[case("git branch 0D foo", OUTPUT_BRANCH_EXISTS, vec!["git branch -D 0D", "git branch -D foo"])]
    #[case("git branch 0l 'maint-*'", OUTPUT_BRANCH_EXISTS, vec!["git branch -D 0l", "git branch -l 'maint-*'"])]
    #[case("git branch 0u upstream", OUTPUT_BRANCH_EXISTS, vec!["git branch -D 0u", "git branch -u upstream"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }

    #[rstest]
    #[case("git branch 0l 'maint-*'", OUTPUT_NOT_VALID_OBJECT, vec!["-l 'maint-*'"])]
    #[case("git branch 0u upstream", OUTPUT_NOT_VALID_OBJECT, vec!["-u upstream"])]
    fn test_get_new_command_not_valid_object(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        let new_command = expected
            .iter()
            .map(|s| format!("git branch {}", s))
            .collect::<Vec<_>>();
        assert_eq!(
            get_new_command(&mut command, None),
            new_command
        );
    }
}
