use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell};

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    command.script.contains('\'') && command.script.contains('\"')
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command.script.replace('\'', "\"")]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "quotation_marks".to_owned(),
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
    #[case(r#"git commit -m 'My Message""#, "", true)]
    #[case("git commit -am \"Mismatched Quotation Marks'", "", true)]
    #[case("echo \"hello'", "", true)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git commit -m 'My Message\"", "", vec!["git commit -m \"My Message\""])]
    #[case("git commit -am \"Mismatched Quotation Marks'", "", vec!["git commit -am \"Mismatched Quotation Marks\""])]
    #[case("echo \"hello'", "", vec!["echo \"hello\""])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
