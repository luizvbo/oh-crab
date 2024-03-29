use super::{
    get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule,
};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;
use std::path::Path;

fn _is_recursive(part: &str) -> bool {
    if part == "--recurse" {
        return true;
    } else if !part.starts_with("--") && part.starts_with("-") && part.contains("r") {
        return true;
    }
    false
}

fn _isdir(part: &str) -> bool {
    !part.starts_with("-") && Path::new(part).exists()
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script_parts.first().map_or(false, |s| s == "prove")
            && output.contains("NOTESTS")
            && !command.script_parts[1..]
                .iter()
                .any(|part| _is_recursive(part))
            && command.script_parts[1..].iter().any(|part| _isdir(part))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["prove"], None),
        command,
    )
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let mut parts = command.script_parts.clone();
    parts.insert(1, "-r".to_owned());
    vec![parts.join(" ")]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "prove_recursively".to_owned(),
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

    const OUTPUT_NOTESTS: &str =
        "Files=0, Tests=0,  0 wallclock secs ( 0.00 usr +  0.00 sys =  0.00 CPU)\nResult: NOTESTS";

    #[rstest]
    #[case("prove -lv t", OUTPUT_NOTESTS, true)]
    #[case("prove app/t", OUTPUT_NOTESTS, true)]
    #[case("prove -lv t", OUTPUT_NOTESTS, false)]
    #[case("prove -r t", OUTPUT_NOTESTS, true)]
    #[case("prove --recurse t", OUTPUT_NOTESTS, true)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("prove -lv t", OUTPUT_NOTESTS, vec!["prove -r -lv t"])]
    #[case("prove t", OUTPUT_NOTESTS, vec!["prove -r t"])]
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
