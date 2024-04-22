use crate::{
    cli::command::CrabCommand,
    rules::{match_rule_with_git_support, utils::git::get_new_command_with_git_support},
    Rule,
};
use shell::Shell;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    command.script.contains(" rm ")
        && command.output.as_ref().map_or(false, |o| {
            o.contains("error: the following file has local modifications")
                && o.contains("use --cached to keep the file, or -f to force removal")
        })
}

pub fn match_rule(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    let mut command_parts = command.script.split_whitespace().collect::<Vec<&str>>();
    if let Some(index) = command_parts.iter().position(|&r| r == "rm") {
        let mut command_list = Vec::new();
        command_parts.insert(index + 1, "--cached");
        command_list.push(command_parts.join(" "));
        command_parts[index + 1] = "-f";
        command_list.push(command_parts.join(" "));
        return command_list;
    }
    vec![]
}

pub fn get_new_command(
    command: &mut CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
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
    use crate::shell::Bash;
    use rstest::rstest;

    #[rstest]
    #[case("git rm foo", "error: the following file has local modifications:\n    foo\n(use --cached to keep the file, or -f to force removal)", true)]
    #[case("git rm foo bar", "error: the following file has local modifications:\n    bar\n(use --cached to keep the file, or -f to force removal)", true)]
    #[case("git rm foo", "", false)]
    #[case("git rm foo bar", "", false)]
    #[case("git rm", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git rm foo", "error: the following file has local modifications:\n    foo\n(use --cached to keep the file, or -f to force removal)", vec!["git rm --cached foo", "git rm -f foo"])]
    #[case("git rm foo bar", "error: the following file has local modifications:\n    bar\n(use --cached to keep the file, or -f to force removal)", vec!["git rm --cached foo bar", "git rm -f foo bar"])]
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
