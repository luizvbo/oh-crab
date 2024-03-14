use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell};

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    command.script_parts.len() >= 2 && command.script_parts[0] == command.script_parts[1]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command.script_parts[1..].join(" ")]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "dry".to_owned(),
        None,
        Some(900),
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
    #[case("cd cd foo", "", true)]
    #[case("git git push origin/master", "", true)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("cd cd foo", "", vec!["cd foo"])]
    #[case("git git push origin/master", "", vec!["git push origin/master"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
