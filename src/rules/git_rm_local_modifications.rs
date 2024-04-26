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
        command.script.contains(" rm ")
            && stdout.contains("error: the following file has local modifications")
            && stdout.contains("use --cached to keep the file, or -f to force removal")
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
    if let Some(index) = command.script_parts.iter().position(|r| r == "rm") {
        let mut command_parts = command.script_parts.clone();
        command_parts.insert(index + 1, "--cached".to_owned());
        let mut command_list = vec![command_parts.join(" ")];
        command_parts[index + 1] = "-f".to_string();
        command_list.push(command_parts.join(" "));
        return command_list;
    }
    vec![]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_rm_local_modifications".to_owned(),
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

    fn output(target: &str) -> String {
        format!("error: the following file has local modifications:\n    {}\n(use --cached to keep the file, or -f to force removal)", target)
    }

    #[rstest]
    #[case("git rm foo", &output("foo"), true)]
    #[case("git rm foo bar", &output("bar"), true)]
    #[case("git rm foo", "", false)]
    #[case("git rm foo bar", "", false)]
    #[case("git rm", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git rm foo", &output("foo"), vec!["git rm --cached foo", "git rm -f foo"])]
    #[case("git rm foo bar", &output("bar"), vec!["git rm --cached foo bar", "git rm -f foo bar"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
