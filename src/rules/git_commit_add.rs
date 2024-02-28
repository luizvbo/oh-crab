use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};

use super::{utils::git::get_new_command_with_git_support, Rule};

use crate::utils::replace_argument;
fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        stdout.contains("no changes added to commit")
            && command.script_parts.contains(&"commit".to_owned())
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
    let opts = vec!["-a", "-p"];
    let mut new_commands = Vec::new();

    for opt in opts {
        let new_command = replace_argument(&command.script, "commit", &format!("commit {}", opt));
        new_commands.push(new_command);
    }
    new_commands
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_commit_add".to_owned(),
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
    #[case("git commit -m \"test\"", "no changes added to commit")]
    #[case("git commit", "no changes added to commit")]
    fn test_match(#[case] script: &str, #[case] output: String) {
        let crab_command = &mut CrabCommand::new(script.to_owned(), Some(output), None);
        assert!(match_rule(crab_command, None));
    }

    #[rstest]
    #[case(
        "git commit -m \"test\"",
        " 1 file changed, 15 insertions(+), 14 deletions(-)"
    )]
    #[case("git branch foo", "")]
    #[case("git checkout feature/test_commit", "")]
    #[case("git push", "")]
    fn test_not_match(#[case] script: &str, #[case] output: String) {
        let crab_command = &mut CrabCommand::new(script.to_owned(), Some(output), None);
        assert!(!match_rule(crab_command, None));
    }

    #[rstest]
    #[case("git commit", vec!["git commit -a", "git commit -p"])]
    #[case("git commit -m \"foo\"", vec!["git commit -a -m \"foo\"", "git commit -p -m \"foo\""])]
    fn test_get_new_command(#[case] script: &str, #[case] new_command: Vec<&str>) {
        let crab_command = &mut CrabCommand::new(script.to_owned(), None, None);
        assert_eq!(get_new_command(crab_command, None), new_command);
    }
}
