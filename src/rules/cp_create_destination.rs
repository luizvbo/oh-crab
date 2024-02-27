use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

pub fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        println!("{:?}", command);
        println!("{:?}", output.contains("No such file or directory"));
        output.contains("No such file or directory")
            || (output.starts_with("cp: directory")
                && output.trim_end().ends_with("does not exist"))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["cp", "mv"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![system_shell.unwrap().and(vec![
        &format!("mkdir -p {}", command.script_parts.last().unwrap()),
        &command.script,
    ])]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cp_create_destination".to_owned(),
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
    #[case("cp", "cp: directory foo does not exist\n", true)]
    #[case("mv", "No such file or directory", true)]
    #[case("cp", "", false)]
    #[case("mv", "", false)]
    #[case("ls", "No such file or directory", false)]
    fn test_match(#[case] script: &str, #[case] output: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(script.to_owned(), Some(output.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case(
        "cp foo bar/",
        "cp: directory foo does not exist\n",
        "mkdir -p bar/ && cp foo bar/"
    )]
    #[case(
        "mv foo bar/",
        "No such file or directory",
        "mkdir -p bar/ && mv foo bar/"
    )]
    #[case(
        "cp foo bar/baz/",
        "cp: directory foo does not exist\n",
        "mkdir -p bar/baz/ && cp foo bar/baz/"
    )]
    fn test_get_new_command(#[case] script: &str, #[case] output: &str, #[case] expected: &str) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(script.to_owned(), Some(output.to_owned()), None);
        assert_eq!(
            get_new_command(&mut command, Some(&system_shell)),
            vec![expected]
        );
    }
}
