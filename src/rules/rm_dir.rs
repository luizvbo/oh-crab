use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

use super::{get_new_command_without_sudo, match_rule_without_sudo, Rule};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        command.script.contains("rm ") && stdout.to_lowercase().contains("is a directory")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let re = Regex::new(r"\brm (.*)").unwrap();
    let repl = |caps: &regex::Captures| {
        let arguments = if command.script.contains("hdfs") {
            "-r"
        } else {
            "-rf"
        };
        format!("rm {} {}", arguments, &caps[1])
    };
    vec![re.replace(&command.script, repl).to_string()]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "rm_dir".to_owned(),
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
    #[case("rm foo", "rm: foo: is a directory", true)]
    #[case("rm foo", "rm: foo: Is a directory", true)]
    #[case("hdfs dfs -rm foo", "rm: `foo`: Is a directory", true)]
    #[case("./bin/hdfs dfs -rm foo", "rm: `foo`: Is a directory", true)]
    #[case("rm foo", "", false)]
    #[case("hdfs dfs -rm foo", "", false)]
    #[case("./bin/hdfs dfs -rm foo", "", false)]
    #[case("", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("rm foo", "", vec!["rm -rf foo"])]
    #[case("hdfs dfs -rm foo", "", vec!["hdfs dfs -rm -r foo"])]
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
