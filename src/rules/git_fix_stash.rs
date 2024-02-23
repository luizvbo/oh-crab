use super::{utils::git::get_command_with_git_support, Rule};
use crate::utils::replace_argument;
use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
    utils::get_closest,
};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        command.script_parts.len() > 1
            && command.script_parts[1] == "stash"
            && stdout.contains("usage:")
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
    if command.script_parts.len() > 2 {
        let stash_commands = vec![
            "apply", "branch", "clear", "drop", "list", "pop", "save", "show",
        ];
        let stash_cmd = &command.script_parts[2];
        println!("{:?}",stash_cmd);
        let fixed = get_closest(stash_cmd, &stash_commands, None, false);
        println!("{:?}",fixed);

        if let Some(fixed) = fixed {
            vec![replace_argument(&command.script, stash_cmd, fixed)]
        } else {
            let mut cmd = command.script_parts.clone();
            cmd.insert(2, "save".to_owned());
            vec![cmd.join(" ")]
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
        "git_fix_stash".to_owned(),
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

    const GIT_STASH_ERR: &str = "\
usage: git stash list [<options>]
   or: git stash show [<stash>]
   or: git stash drop [-q|--quiet] [<stash>]
   or: git stash ( pop | apply ) [--index] [-q|--quiet] [<stash>]
   or: git stash branch <branchname> [<stash>]
   or: git stash [save [--patch] [-k|--[no-]keep-index] [-q|--quiet]
\t\t       [-u|--include-untracked] [-a|--all] [<message>]]
   or: git stash clear
";

    #[rstest]
    #[case("git stash opp", GIT_STASH_ERR, true)]
    #[case("git stash Some message", GIT_STASH_ERR, true)]
    #[case("git stash saev Some message", GIT_STASH_ERR, true)]
    #[case("git", GIT_STASH_ERR, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("git stash opp", GIT_STASH_ERR, vec!["git stash pop"])]
    #[case("git stash Some message", GIT_STASH_ERR, vec!["git stash save Some message"])]
    #[case("git stash saev Some message", GIT_STASH_ERR, vec!["git stash save Some message"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
