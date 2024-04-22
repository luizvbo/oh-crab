use crate::{
    cli::command::CrabCommand,
    rules::{
        utils::git::{get_new_command_with_git_support, match_rule_with_git_support},
        Rule,
    },
    shell::Shell,
};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        let re = Regex::new(r"src refspec \w+ does not match any").unwrap();
        re.is_match(stdout)
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, _system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    vec![system_shell
        .unwrap()
        .and(vec!["git commit -m \"Initial commit\"", &command.script])]
}

pub fn get_new_command(
    command: &mut CrabCommand,
    _system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, _system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_push_without_commits".to_owned(),
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
    #[case(
        "git push -u origin master",
        "error: src refspec master does not match any\nerror: failed to...",
        true
    )]
    #[case("git push -u origin master", "Everything up-to-date", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git push -u origin master", "error: src refspec master does not match any\nerror: failed to...", vec!["git commit -m \"Initial commit\" && git push -u origin master"])]
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
