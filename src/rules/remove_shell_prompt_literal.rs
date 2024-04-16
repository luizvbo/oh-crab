use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(output) = &command.output {
        let re = Regex::new(r"^[\s]*\$ [\S]+").unwrap();
        output.contains("$: command not found") && re.is_match(&command.script)
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command
        .script
        .trim_start_matches(|c: char| c == '$' || c.is_whitespace())
        .to_owned()]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "remove_shell_prompt_literal".to_owned(),
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
    #[case("$ cd newdir", "$: command not found", true)]
    #[case(" $ cd newdir", "$: command not found", true)]
    #[case("$ $ cd newdir", "$: command not found", true)]
    #[case(" $ $ cd newdir", "$: command not found", true)]
    #[case("$", "$: command not found", false)]
    #[case(" $", "$: command not found", false)]
    #[case("$?", "127: command not found", false)]
    #[case(" $?", "127: command not found", false)]
    #[case("", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("$ cd newdir", "", vec!["cd newdir"])]
    #[case("$ $ cd newdir", "", vec!["cd newdir"])]
    #[case("$ python3 -m virtualenv env", "", vec!["python3 -m virtualenv env"])]
    #[case(" $ $ $ python3 -m virtualenv env", "", vec!["python3 -m virtualenv env"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
