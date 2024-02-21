use crate::{
    cli::command::CrabCommand,
    rules::utils::git::match_rule_with_git_support,
    shell::Shell,
    utils::{get_all_matched_commands, replace_command},
};

use super::{utils::git::get_command_with_git_support, Rule};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        stdout.contains(" is not a git command. See 'git --help'.")
            && (stdout.contains("The most similar command") || stdout.contains("Did you mean"))
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
    if let Some(stdout) = &command.stdout {
        let re = Regex::new(r"git: '([^']*)' is not a git command").unwrap();

        let broken_cmd = match re.captures(stdout) {
            Some(caps) => caps.get(1).map_or("", |m| m.as_str()),
            None => "",
        };
        let matched = get_all_matched_commands(
            stdout,
            Some(vec!["The most similar command", "Did you mean"]),
        );
        replace_command(
            command,
            broken_cmd,
            matched.iter().map(|s| s.as_str()).collect(),
        )
    } else {
        Vec::<String>::new()
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_not_command".to_owned(),
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

    const GIT_NOT_COMMAND: &str = r#"git: 'brnch' is not a git command. See 'git --help'.

The most similar command is
branch
"#;
    const GIT_NOT_COMMAND_ONE_OF_THIS: &str = r#"git: 'st' is not a git command. See 'git --help'.

The most similar commands are
status
reset
stage
stash
stats
"#;
    const GIT_NOT_COMMAND_CLOSEST: &str = r#"
git: 'tags' is not a git command. See 'git --help'.

The most similar commands are
    stage
    tag
"#;
    const GIT_COMMAND: &str = "* master";

    #[rstest]
    #[case("git brnch", GIT_NOT_COMMAND.to_owned(), true)]
    #[case("git st", GIT_NOT_COMMAND_ONE_OF_THIS.to_owned(), true)]
    #[case("ls brnch", GIT_NOT_COMMAND.to_owned(), false)]
    #[case("git branch", GIT_COMMAND.to_owned(), false)]
    fn test_match(#[case] script: &str, #[case] output: String, #[case] is_match: bool) {
        let crab_command = &mut CrabCommand::new(script.to_owned(), Some(output), None);
        assert_eq!(match_rule(crab_command, None), is_match);
    }

    #[rstest]
    #[case("git brnch", GIT_NOT_COMMAND.to_owned(), vec!["git branch"])]
    #[case("git st", GIT_NOT_COMMAND_ONE_OF_THIS.to_owned(), vec!["git reset", "git stage", "git stash"])]
    #[case("git tags", GIT_NOT_COMMAND_CLOSEST.to_owned(), vec!["git tag", "git stage"])]
    fn test_get_new_command(
        #[case] script: &str,
        #[case] output: String,
        #[case] new_command: Vec<&str>,
    ) {
        let crab_command = &mut CrabCommand::new(script.to_owned(), Some(output), None);
        assert_eq!(get_new_command(crab_command, None), new_command);
    }
}
