use crate::{cli::command::CrabCommand, shell::Shell};

use super::Rule;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    println!("{:?}", command);
    if let Some(output) = &command.output {
        output.ends_with("run ag with -Q\n")
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command.script.replacen("ag", "ag -Q", 1)]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "ag_literal".to_owned(),
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

    const STDERR: &str = "ERR: Bad regex! pcre_compile() failed at position 1: missing )\n\
        If you meant to search for a literal string, run ag with -Q\n";

    #[rstest]
    #[case("ag \\(", STDERR, true)]
    #[case("ag foo", "", false)]
    fn test_match(#[case] command: &str, #[case] stderr: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), None, Some(stderr.to_owned()));
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("ag \\(", vec!["ag -Q \\("])]
    fn test_get_new_command(#[case] command: &str, #[case] expected: Vec<&str>) {
        let mut command = CrabCommand::new(command.to_owned(), None, Some(STDERR.to_owned()));
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
