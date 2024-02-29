use crate::{cli::command::CrabCommand, shell::Shell};

use super::{get_new_command_without_sudo, match_rule_without_sudo, Rule};

use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script_parts.first().map_or(false, |s| s == "ln")
            && output.ends_with("hard link not allowed for directory")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(auxiliary_match_rule, command)
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let re = Regex::new(r"^ln ").unwrap();
    vec![re.replace_all(&command.script, "ln -s ").into_owned()]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "ln_no_hard_link".to_owned(),
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
        "ln barDir barLink",
        "ln: ‘barDir’: hard link not allowed for directory",
        true
    )]
    #[case("sudo ln a b", "ln: ‘a’: hard link not allowed for directory", true)]
    #[case(
        "sudo ln -nbi a b",
        "ln: ‘a’: hard link not allowed for directory",
        true
    )]
    #[case("", "", false)]
    #[case("ln a b", "... hard link", false)]
    #[case("sudo ln a b", "... hard link", false)]
    #[case("a b", "hard link not allowed for directory", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("ln barDir barLink", "", vec!["ln -s barDir barLink"])]
    #[case("sudo ln barDir barLink", "", vec!["sudo ln -s barDir barLink"])]
    #[case("sudo ln -nbi a b", "", vec!["sudo ln -s -nbi a b"])]
    #[case("ln -nbi a b && ls", "", vec!["ln -s -nbi a b && ls"])]
    #[case("ln a ln", "", vec!["ln -s a ln"])]
    #[case("sudo ln a ln", "", vec!["sudo ln -s a ln"])]
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
