use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

use super::{get_new_command_without_sudo, match_rule_without_sudo, Rule};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script.contains("mkdir") && output.contains("No such file or directory")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let re = Regex::new(r"\bmkdir (.*)").unwrap();
    vec![re.replace_all(&command.script, "mkdir -p $1").to_string()]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "mkdir_p.rs".to_owned(),
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
    #[case("mkdir foo/bar/baz", "mkdir: foo/bar: No such file or directory", true)]
    #[case(
        "./bin/hdfs dfs -mkdir foo/bar/baz",
        "mkdir: `foo/bar/baz\': No such file or directory",
        true
    )]
    #[case(
        "hdfs dfs -mkdir foo/bar/baz",
        "mkdir: `foo/bar/baz\': No such file or directory",
        true
    )]
    #[case("mkdir foo/bar/baz", "", false)]
    #[case("mkdir foo/bar/baz", "foo bar baz", false)]
    #[case("hdfs dfs -mkdir foo/bar/baz", "", false)]
    #[case("./bin/hdfs dfs -mkdir foo/bar/baz", "", false)]
    #[case("", "", false)]
    fn test_match(#[case] command: &str, #[case] output: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(output.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("mkdir foo/bar/baz", "", "mkdir -p foo/bar/baz")]
    #[case("hdfs dfs -mkdir foo/bar/baz", "", "hdfs dfs -mkdir -p foo/bar/baz")]
    #[case(
        "./bin/hdfs dfs -mkdir foo/bar/baz",
        "",
        "./bin/hdfs dfs -mkdir -p foo/bar/baz"
    )]
    fn test_get_new_command(#[case] command: &str, #[case] output: &str, #[case] expected: &str) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(output.to_owned()), None);
        assert_eq!(
            get_new_command(&mut command, Some(&system_shell)),
            vec![expected]
        );
    }
}
