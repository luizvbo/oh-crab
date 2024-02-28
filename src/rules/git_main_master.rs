use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};

use super::{utils::git::get_new_command_with_git_support, Rule};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        stdout.contains("'master'") || stdout.contains("'main'")
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
    if let Some(stdout) = &command.output {
        if stdout.contains("'master'") {
            vec![command.script.replace("master", "main")]
        } else {
            vec![command.script.replace("main", "master")]
        }
    } else {
        Vec::<String>::new()
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_main_master".to_owned(),
        None,
        Some(1200),
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

    fn output_branch(branch_name: Option<&str>) -> String {
        if let Some(branch_name) = branch_name {
            format!(
                "error: pathspec '{}' did not match any file(s) known to git",
                branch_name
            )
        } else {
            "".to_owned()
        }
    }

    #[rstest]
    #[case("git checkout main", Some("main"), true)]
    #[case("git checkout master", Some("master"), true)]
    #[case("git show main", Some("main"), true)]
    #[case("git checkout master", None, false)]
    #[case("git checkout main", None, false)]
    #[case("git checkout wibble", Some("wibble"), false)]
    fn test_match(
        #[case] command: &str,
        #[case] branch_name: Option<&str>,
        #[case] is_match: bool,
    ) {
        let mut command =
            CrabCommand::new(command.to_owned(), Some(output_branch(branch_name)), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git checkout main", Some("main"), vec!["git checkout master"])]
    #[case("git checkout master", Some("master"), vec!["git checkout main"])]
    #[case("git checkout main", Some("main"), vec!["git checkout master"])]
    #[case("git checkout wibble", Some("wibble"),  vec!["git checkout wibble"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] branch_name: Option<&str>,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command =
            CrabCommand::new(command.to_owned(), Some(output_branch(branch_name)), None);
        assert_eq!(get_new_command(&mut command, Some(&system_shell)), expected);
    }
}
