use super::{utils::git::get_command_with_git_support, Rule};
use crate::utils::{replace_command};
use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        command.script.contains("bisect") && stdout.contains("usage: git bisect")
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
        let re_broken = Regex::new(r"git bisect ([^ $]*).*").unwrap();
        let re_usage = Regex::new(r"usage: git bisect \[([^\]]+)\]").unwrap();

        let broken = re_broken.captures(&command.script);
        let usage = re_broken.captures(stdout);
        if broken.is_some() && usage.is_some() {
            replace_command(
                command,
                &broken.unwrap()[1],
                usage.unwrap()[1].split('|').collect(),
            )
        } else {
            Vec::<String>::new()
        }
    } else {
        Vec::<String>::new()
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_merge".to_owned(),
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

    const OUTPUT: &str = "usage: git bisect [help|start|bad|good|new|old|terms|skip|next|reset|visualize|replay|log|run]";

    use rstest::rstest;

    #[rstest]
    #[case("git bisect strt", OUTPUT, true)]
    #[case("git bisect rset", OUTPUT, true)]
    #[case("git bisect goood", OUTPUT, true)]
    #[case("git bisect", "", false)]
    #[case("git bisect start", "", false)]
    #[case("git bisect good", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git bisect goood", OUTPUT, vec!["good", "old", "bad"])]
    #[case("git bisect strt", OUTPUT, vec!["start", "next", "skip"])]
    #[case("git bisect rset", OUTPUT, vec!["reset", "new", "next"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        let new_command = expected.iter().map(|s| format!("git bisect {}", s)).collect::<Vec<_>>();
        assert_eq!(get_new_command(&mut command, Some(&system_shell)), new_command);
    }
}
