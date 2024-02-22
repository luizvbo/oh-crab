use super::{utils::git::get_command_with_git_support, Rule};
use crate::utils::get_close_matches;

use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        command.script.contains("rebase")
            && stdout.contains("It seems that there is already a rebase-merge directory")
            && stdout.contains("I wonder if you are in the middle of another rebase")
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
        let rm_cmd_split: Vec<&str> = stdout.split("\n").collect();
        if rm_cmd_split.len() > 4 {
            let command_list = vec![
                "git rebase --continue",
                "git rebase --abort",
                "git rebase --skip",
                rm_cmd_split[rm_cmd_split.len() - 4].trim(),
            ];
            get_close_matches(&command.script, &command_list, Some(4), Some(0.))
                .iter()
                .map(|s| s.to_string())
                .collect()
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
        "git_help_aliased".to_owned(),
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

    const OUTPUT: &str = "\n\nIt seems that there is already a rebase-merge directory, and\n\
        I wonder if you are in the middle of another rebase.  If that is the\n\
        case, please try\n\tgit rebase (--continue | --abort | --skip)\n\
        If that is not the case, please\n\trm -fr \"/foo/bar/baz/egg/.git/rebase-merge\"\n\
        and run me again.  I am stopping in case you still have something\nvaluable there.\n";

    #[rstest]
    #[case("git rebase master", OUTPUT, true)]
    #[case("git rebase -skip", OUTPUT, true)]
    #[case("git rebase", OUTPUT, true)]
    #[case("git rebase master", "", false)]
    #[case("git rebase -abort", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git rebase master", OUTPUT, vec!["git rebase --abort", "git rebase --skip", "git rebase --continue", "rm -fr \"/foo/bar/baz/egg/.git/rebase-merge\""])]
    #[case("git rebase -skip", OUTPUT, vec!["git rebase --skip", "git rebase --abort", "git rebase --continue", "rm -fr \"/foo/bar/baz/egg/.git/rebase-merge\""])]
    #[case("git rebase", OUTPUT, vec!["git rebase --skip", "git rebase --abort", "git rebase --continue", "rm -fr \"/foo/bar/baz/egg/.git/rebase-merge\""])]
    fn test_get_new_command(#[case] command: &str, #[case] stdout: &str, #[case] expected: Vec<&str>) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
