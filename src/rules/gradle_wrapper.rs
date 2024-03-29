use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use std::path::Path;
use which::which;

fn gradlew_exists() -> bool {
    Path::new("gradlew").exists()
}

fn is_terminal_command(terminal_command: &str) -> bool {
    which(terminal_command).is_ok()
}

fn auxiliary_match_rule<F, G>(
    command: &CrabCommand,
    fn_is_terminal_command: F,
    fn_file_exists: G,
) -> bool
where
    F: Fn(&str) -> bool,
    G: Fn() -> bool,
{
    if let Some(output) = &command.output {
        !fn_is_terminal_command(&command.script_parts[0])
            && output.contains("not found")
            && fn_file_exists()
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(
        |command| auxiliary_match_rule(command, is_terminal_command, gradlew_exists),
        command,
        vec!["gradle"],
        None,
    )
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![format!("./gradlew {}", command.script_parts[1..].join(" "))]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "gradle_wrapper".to_owned(),
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
    use super::{auxiliary_match_rule, get_new_command};
    use crate::cli::command::CrabCommand;
    use crate::rules::utils::match_rule_with_is_app;
    use crate::shell::Bash;
    use rstest::rstest;

    #[rstest]
    #[case("gradle tasks", "gradle: not found", false, true, true)]
    #[case("gradle build", "gradle: not found", false, true, true)]
    #[case("gradle tasks", "gradle: not found", false, false, false)]
    #[case("gradle tasks", "command not found", true, true, false)]
    #[case("npm tasks", "npm: not found", false, true, false)]
    fn test_match(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] is_terminal_command: bool,
        #[case] file_exists: bool,
        #[case] is_match: bool,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(
            match_rule_with_is_app(
                |command| auxiliary_match_rule(command, |s| is_terminal_command, || file_exists),
                &command,
                vec!["gradle"],
                None,
            ),
            is_match
        );
    }

    #[rstest]
    #[case("gradle assemble", "", vec!["./gradlew assemble"])]
    #[case("gradle --help", "", vec!["./gradlew --help"])]
    #[case("gradle build -c", "", vec!["./gradlew build -c"])]
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
