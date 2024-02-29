use crate::{cli::command::CrabCommand, shell::Shell};

use super::{utils::match_rule_with_is_app, Rule};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    !command.script_parts.is_empty() && !command.script.contains("ls -l")
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["ls"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let mut script_parts = command.script_parts.clone();
    script_parts[0] = "ls -lah".to_string();
    vec![script_parts.join(" ")]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "ls_lah".to_owned(),
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
    #[case("ls", "", true)]
    #[case("ls file.py", "", true)]
    #[case("ls /opt", "", true)]
    #[case("ls -lah /opt", "", false)]
    #[case("pacman -S binutils", "", false)]
    #[case("lsof", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("ls file.py", "", vec!["ls -lah file.py"])]
    #[case("ls", "", vec!["ls -lah"])]
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
