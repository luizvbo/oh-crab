use super::{
    get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule,
};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script_parts.first().map_or(false, |s| s == "cp")
            && (output.contains("omitting directory") || output.contains("is a directory"))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["cp"], None),
        command,
    )
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let re = Regex::new(r"^cp ").unwrap();
    vec![re.replace_all(&command.script, "cp -a ").into_owned()]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cp_omitting_directory".to_owned(),
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
    #[case("cp dir", "cp: dor: is a directory", true)]
    #[case("cp dir", "cp: omitting directory 'dir'", true)]
    #[case("some dir", "cp: dor: is a directory", false)]
    #[case("some dir", "cp: omitting directory 'dir'", false)]
    #[case("cp dir", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("cp dir", "", vec!["cp -a dir"])]
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
