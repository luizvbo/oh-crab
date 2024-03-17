use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

use super::{
    get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule,
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        let stdout = stdout.to_lowercase();
        command.script.starts_with("cd ")
            && (stdout.contains("no such file or directory")
                || stdout.contains("cd: can't cd to")
                || stdout.contains("does not exist"))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["cd"], None),
        command,
    )
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let re = Regex::new(r"^cd (.*)").unwrap();
    let repl = |caps: &regex::Captures| format!("mkdir -p {} && cd {}", &caps[1], &caps[1]);
    vec![re.replace(&command.script, repl).to_string()]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cd_mkdir".to_owned(),
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
    #[case("cd foo", "cd: foo: No such file or directory", true)]
    #[case("cd foo/bar/baz", "cd: foo: No such file or directory", true)]
    #[case("cd foo/bar/baz", "cd: can't cd to foo/bar/baz", true)]
    #[case("cd /foo/bar/", "cd: The directory \"/foo/bar/\" does not exist", true)]
    #[case("cd foo", "", false)]
    #[case("", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("cd foo", "", vec!["mkdir -p foo && cd foo"])]
    #[case("cd foo/bar/baz", "", vec!["mkdir -p foo/bar/baz && cd foo/bar/baz"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
