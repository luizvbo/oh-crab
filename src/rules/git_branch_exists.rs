use super::{utils::git::get_command_with_git_support, Rule};
use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        stdout.contains("fatal: A branch named '") && stdout.contains("' already exists.")
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
        let re_branch_name = Regex::new(r"fatal: A branch named '(.+)' already exists.").unwrap();
        if let Some(captures) = re_branch_name.captures(stdout) {
            let mut new_commands = Vec::<String>::new();
            let branch_name = &captures[1].replace('\'', r"\'");
            let new_command_templates = vec![
                vec!["git branch -d", "git branch"],
                vec!["git branch -d", "git checkout -b"],
                vec!["git branch -D", "git branch"],
                vec!["git branch -D", "git checkout -b"],
                vec!["git checkout"],
            ];
            for new_command_template in new_command_templates {
                let new_command_with_branch = new_command_template
                    .iter()
                    .map(|s| format!("{} {}", s, branch_name))
                    .collect::<Vec<String>>();
                new_commands.push(
                    system_shell
                        .unwrap()
                        .and(new_command_with_branch.iter().map(AsRef::as_ref).collect()),
                );
            }
            new_commands
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
        "git_branch_exists".to_owned(),
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

    const OUTPUT: &str = "fatal: A branch named '#' already exists.";

    use rstest::rstest;

    #[rstest]
    #[case("git branch foo", "foo", OUTPUT)]
    #[case("git checkout bar", "bar", OUTPUT)]
    #[case("git checkout -b \"let's-push-this\"", "\"let's-push-this\"", OUTPUT)]
    fn test_match(
        #[case] command: &str,
        #[case] src_branch_name: &str,
        #[case] stdout_template: &str,
    ) {
        let stdout = stdout_template.replace('#', src_branch_name);
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout), None);
        assert!(match_rule(&mut command, None));
    }

    #[rstest]
    #[case("git branch foo")]
    #[case("git checkout bar")]
    #[case("git checkout -b \"let's-push-this\"")]
    fn test_not_match(#[case] command: &str) {
        let mut command = CrabCommand::new(command.to_owned(), Some("".to_owned()), None);
        assert!(!match_rule(&mut command, None));
    }

    #[rstest]
    #[case("git branch foo", "foo", "foo", OUTPUT)]
    #[case("git checkout bar", "bar", "bar", OUTPUT)]
    #[case(
        "git checkout -b \"let's-push-this\"",
        "let's-push-this",
        "let\\'s-push-this",
        OUTPUT
    )]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] src_branch_name: &str,
        #[case] branch_name: &str,
        #[case] stdout_template: &str,
    ) {
        let expected: Vec<String> = [
            "git branch -d # && git branch #",
            "git branch -d # && git checkout -b #",
            "git branch -D # && git branch #",
            "git branch -D # && git checkout -b #",
            "git checkout #",
        ]
        .iter()
        .map(|s| s.replace('#', branch_name))
        .collect();
        let stdout = stdout_template.replace('#', src_branch_name);
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, Some(&system_shell)), expected);
    }
}
