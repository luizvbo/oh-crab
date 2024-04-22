
use crate::{
    cli::command::CrabCommand,
    rules::{utils::git::get_new_command_with_git_support, match_rule_with_git_support},
    Rule,
};
use regex::Regex;
use shell::Shell;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    let error_pattern = Regex::new(r"fatal: bad flag '(.*?)' used after filename").unwrap();
    let error_pattern2 = Regex::new(r"fatal: option '(.*?)' must come before non-option arguments").unwrap();
    error_pattern.is_match(&command.output) || error_pattern2.is_match(&command.output)
}

pub fn match_rule(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(command: &CrabCommand, _system_shell: Option<&dyn Shell>) -> Vec<String> {
    let error_pattern = Regex::new(r"fatal: bad flag '(.*?)' used after filename").unwrap();
    let error_pattern2 = Regex::new(r"fatal: option '(.*?)' must come before non-option arguments").unwrap();
    let mut command_parts = command.script.split_whitespace().collect::<Vec<&str>>();

    if let Some(caps) = error_pattern.captures(&command.output).or_else(|| error_pattern2.captures(&command.output)) {
        if let Some(bad_flag) = caps.get(1) {
            let bad_flag = bad_flag.as_str();
            if let Some(bad_flag_index) = command_parts.iter().position(|&r| r == bad_flag) {
                for index in (0..bad_flag_index).rev() {
                    if !command_parts[index].starts_with('-') {
                        command_parts.swap(bad_flag_index, index);
                        break;
                    }
                }
            }
        }
    }

    vec![command_parts.join(" ")]
}

pub fn get_new_command(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_flag_after_filename".to_owned(),
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
    #[case("git log README.md -p", "fatal: bad flag '-p' used after filename", true)]
    #[case("git log README.md -p CONTRIBUTING.md", "fatal: bad flag '-p' used after filename", true)]
    #[case("git log -p README.md --name-only", "fatal: bad flag '--name-only' used after filename", true)]
    #[case("git log README.md -p", "fatal: option '-p' must come before non-option arguments", true)]
    #[case("git log README.md -p CONTRIBUTING.md", "fatal: option '-p' must come before non-option arguments", true)]
    #[case("git log -p README.md --name-only", "fatal: option '--name-only' must come before non-option arguments", true)]
    #[case("git log README.md", "", false)]
    #[case("git log -p README.md", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git log README.md -p", "fatal: bad flag '-p' used after filename", vec!["git log -p README.md"])]
    #[case("git log README.md -p CONTRIBUTING.md", "fatal: bad flag '-p' used after filename", vec!["git log -p README.md CONTRIBUTING.md"])]
    #[case("git log -p README.md --name-only", "fatal: bad flag '--name-only' used after filename", vec!["git log -p --name-only README.md"])]
    #[case("git log README.md -p", "fatal: option '-p' must come before non-option arguments", vec!["git log -p README.md"])]
    #[case("git log README.md -p CONTRIBUTING.md", "fatal: option '-p' must come before non-option arguments", vec!["git log -p README.md CONTRIBUTING.md"])]
    #[case("git log -p README.md --name-only", "fatal: option '--name-only' must come before non-option arguments", vec!["git log -p --name-only README.md"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, Some(&system_shell)), expected);
    }
}
