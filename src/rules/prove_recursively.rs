use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use std::path::Path;

fn is_recursive(part: &str) -> bool {
    part == "--recurse" || (!part.starts_with("--") && part.starts_with('-') && part.contains('r'))
}

fn is_dir(part: &str) -> bool {
    !part.starts_with('-') && Path::new(part).exists()
}

fn auxiliary_match_rule<F>(command: &CrabCommand, fn_is_dir: F) -> bool
where
    F: Fn(&str) -> bool,
{
    if let Some(output) = &command.output {
        output.contains("NOTESTS")
            && !command.script_parts[1..]
                .iter()
                .any(|part| is_recursive(part))
            && command.script_parts[1..].iter().any(|part| fn_is_dir(part))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(
        |command| auxiliary_match_rule(command, is_dir),
        command,
        vec!["prove"],
        None,
    )
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let mut parts = command.script_parts.clone();
    parts.insert(1, "-r".to_owned());
    vec![parts.join(" ")]
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
    use super::{auxiliary_match_rule, get_new_command};
    use crate::cli::command::CrabCommand;
    use crate::rules::utils::match_rule_with_is_app;
    use crate::shell::Bash;
    use rstest::rstest;

    const OUTPUT: &str =
        "Files=0, Tests=0,  0 wallclock secs ( 0.00 usr +  0.00 sys =  0.00 CPU)\nResult: NOTESTS";

    #[rstest]
    #[case("prove -lv t", OUTPUT, true, true)]
    #[case("prove app/t", OUTPUT, true, true)]
    #[case("prove -lv t", OUTPUT, false, false)]
    #[case("prove -r t", OUTPUT, true, false)]
    #[case("prove --recurse t", OUTPUT, true, false)]
    fn test_match(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] is_dir: bool,
        #[case] is_match: bool,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(
            match_rule_with_is_app(
                |command| auxiliary_match_rule(command, |s| is_dir),
                &command,
                vec!["prove"],
                None,
            ),
            is_match
        );
    }

    #[rstest]
    #[case("prove -lv t", OUTPUT, vec!["prove -r -lv t"])]
    #[case("prove t", OUTPUT, vec!["prove -r t"])]
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
