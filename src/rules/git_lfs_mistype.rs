use regex::Regex;

use crate::{
    cli::command::CrabCommand,
    rules::{
        utils::git::{get_new_command_with_git_support, match_rule_with_git_support},
        Rule,
    },
    shell::Shell,
    utils::{get_all_matched_commands, replace_command},
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script.contains("lfs") && output.contains("Did you mean this?")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    if let Some(output) = &command.output {
        let re = Regex::new(r#"Error: unknown command "([^"]*)" for "git-lfs""#).unwrap();
        if let Some(caps) = re.captures(output) {
            if let Some(broken_cmd) = caps.get(1) {
                let broken_cmd = broken_cmd.as_str();
                let matched =
                    get_all_matched_commands(output, Some(vec!["Did you mean", " for usage."]));
                return replace_command(
                    command,
                    broken_cmd,
                    matched.iter().map(|s| s.as_str()).collect(),
                );
            }
        }
    }
    vec![]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_lfs_mistype".to_owned(),
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

    const MISTYPE_RESPONSE: &str = "Error: unknown command \"evn\" for \"git-lfs\"\n\nDid you mean this?\n        env\n        ext\n\nRun 'git-lfs --help' for usage.\n";

    #[rstest]
    #[case("git lfs evn", MISTYPE_RESPONSE, true)]
    #[case("git lfs env", "bash: git: command not found", false)]
    #[case("docker lfs env", MISTYPE_RESPONSE, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git lfs evn", MISTYPE_RESPONSE, vec!["git lfs env", "git lfs ext"])]
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
