use super::{
    get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule,
};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script_parts.first().map_or(false, |s| s == "lein")
            && output.contains("is not a task. See 'lein help'")
            && output.contains("Did you mean this?")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["lein"], None),
        command,
    )
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    if let Some(output) = &command.output {
        let re = Regex::new(r"'([^']*)' is not a task").unwrap();
        let broken_cmd = re
            .captures(output)
            .unwrap()
            .get(1)
            .map_or("", |m| m.as_str());
        let new_cmds = output.split("Did you mean this?").collect::<Vec<&str>>();
        new_cmds
            .iter()
            .map(|cmd| command.script.replace(broken_cmd, cmd))
            .collect()
    } else {
        Vec::<String>::new()
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "lein_not_task".to_owned(),
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

    const IS_NOT_TASK: &str =
        "'rpl' is not a task. See 'lein help'.\n\nDid you mean this?\n     repl\n     jar\n";

    #[rstest]
    #[case("lein rpl", IS_NOT_TASK, true)]
    #[case("ls", IS_NOT_TASK, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    // TODO: Fix test issue
    #[case("lein rpl --help", IS_NOT_TASK, vec!["lein repl --help", "lein jar --help"])]
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
