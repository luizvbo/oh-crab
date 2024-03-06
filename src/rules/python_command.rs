use super::{get_new_command_without_sudo, match_rule_without_sudo, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if !command.script_parts.is_empty() {
        if let Some(output) = &command.output {
            command.script_parts[0].ends_with(".py")
                && (output.contains("Permission denied") || output.contains("command not found"))
        } else {
            false
        }
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(auxiliary_match_rule, command)
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    vec!["python ".to_owned() + &command.script]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "python_command".to_owned(),
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
    #[case("temp.py", "Permission denied", true)]
    #[case("temp.py", "[...] command not found [...]", true)]
    #[case("temp.rs", "[...] command not found [...]", false)]
    #[case("", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("./test_sudo.py", "", vec!["python ./test_sudo.py"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
