use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(|command| true, command, vec!["man"], Some(1))
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if command.script.contains('3') {
        return vec![command.script.replace('3', "2")];
    }
    if command.script.contains('2') {
        return vec![command.script.replace('2', "3")];
    }
    if let Some(output) = &command.output {
        let last_arg = command.script_parts.last().unwrap();
        let help_command = format!("{last_arg} --help");

        if output.trim() == format!("No manual entry for {last_arg}") {
            return vec![help_command];
        }

        let mut split_cmd2 = command.script_parts.clone();
        let mut split_cmd3 = split_cmd2.clone();

        split_cmd2.insert(1, " 2 ".to_string());
        split_cmd3.insert(1, " 3 ".to_string());

        vec![split_cmd3.join(""), split_cmd2.join(""), help_command]
    } else {
        Vec::<String>::new()
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "man".to_owned(),
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
    #[case("man read", "", true)]
    #[case("man 2 read", "", true)]
    #[case("man 3 read", "", true)]
    #[case("man -s2 read", "", true)]
    #[case("man -s3 read", "", true)]
    #[case("man -s 2 read", "", true)]
    #[case("man -s 3 read", "", true)]
    #[case("man", "", false)]
    #[case("man ", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("man read", "", vec!["man 3 read", "man 2 read", "read --help"])]
    #[case("man missing", "No manual entry for missing\n", vec!["missing --help"])]
    #[case("man 2 read", "", vec!["man 3 read"])]
    #[case("man 3 read", "", vec!["man 2 read"])]
    #[case("man -s2 read", "", vec!["man -s3 read"])]
    #[case("man -s3 read", "", vec!["man -s2 read"])]
    #[case("man -s 2 read", "", vec!["man -s 3 read"])]
    #[case("man -s 3 read", "", vec!["man -s 2 read"])]
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
