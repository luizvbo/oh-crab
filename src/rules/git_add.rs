use crate::{
    cli::command::CrabCommand,
    rules::{
        utils::git::{get_new_command_with_git_support, match_rule_with_git_support},
        Rule,
    },
    shell::Shell,
};
use regex::Regex;
use shlex::Quoter; // Import the Quoter
use std::path::Path;

fn get_missing_file(command: &CrabCommand, path_exists: Option<bool>) -> Option<String> {
    if let Some(stdout) = &command.output {
        let re = Regex::new(r"error: pathspec '([^']*)' did not match any file\(s\) known to git.")
            .unwrap();

        if let Some(captures) = re.captures(stdout) {
            let path = &captures[1];
            if path.is_empty() {
                None
            } else if let Some(path_exists) = path_exists {
                if path_exists {
                    Some(path.to_owned())
                } else {
                    None
                }
            } else if Path::new(&path).exists() {
                Some(path.to_owned())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn mockable_match_rule(command: &CrabCommand, path_exists: Option<bool>) -> bool {
    if let Some(stdout) = &command.output {
        stdout.contains("did not match any file(s) known to git.")
            && get_missing_file(command, path_exists).is_some()
    } else {
        false
    }
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    mockable_match_rule(command, None)
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn mockable_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
    path_exists: Option<bool>,
) -> Vec<String> {
    let missing_file = get_missing_file(command, path_exists).unwrap_or_default();
    // Use the shlex Quoter to handle spaces and special characters
    let quoter = Quoter::new();
    let quoted_missing_file = quoter.quote(&missing_file).unwrap_or_default();

    let str_git_add = format!("git add -- {}", quoted_missing_file);
    vec![system_shell
        .unwrap()
        .and(vec![&str_git_add, &command.script])]
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    mockable_get_new_command(command, system_shell, None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_add".to_owned(),
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
    use super::{mockable_get_new_command, mockable_match_rule};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;
    use rstest::rstest;

    #[rstest]
    #[case("git submodule update unknown", "unknown", true, true)]
    #[case("git commit unknown", "unknown", true, true)]
    #[case("git submodule update known", "", true, false)]
    #[case("git commit known", "", true, false)]
    fn test_match_rule(
        #[case] script: &str,
        #[case] target: &str,
        #[case] path_exists: bool,
        #[case] expected: bool,
    ) {
        let stdout = if !target.is_empty() {
            format!(
                "error: pathspec '{}' did not match any file(s) known to git.",
                target
            )
        } else {
            "".to_string()
        };
        let command = CrabCommand::new(script.to_owned(), Some(stdout), None);
        assert_eq!(mockable_match_rule(&command, Some(path_exists)), expected);
    }

    #[rstest]
    #[case(
        "git submodule update unknown",
        "unknown",
        "git add -- unknown && git submodule update unknown"
    )]
    #[case(
        "git commit unknown",
        "unknown",
        "git add -- unknown && git commit unknown"
    )]
    // This is the corrected test case:
    #[case(
        "git commit \"file with spaces.txt\"",
        "file with spaces.txt",
        "git add -- 'file with spaces.txt' && git commit \"file with spaces.txt\""
    )]
    fn test_get_new_command(#[case] script: &str, #[case] target: &str, #[case] expected: &str) {
        let stdout = format!(
            "error: pathspec '{}' did not match any file(s) known to git.",
            target
        );
        let system_shell = Bash {};
        let command = CrabCommand::new(script.to_owned(), Some(stdout), None);

        // The assertion now correctly expects single quotes from shlex in the `git add`
        // part and preserves the original double quotes in the `git commit` part.
        assert_eq!(
            mockable_get_new_command(&command, Some(&system_shell), Some(true))[0],
            expected
        );
    }
}
