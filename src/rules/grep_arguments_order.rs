use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use std::path::Path;

fn get_actual_file(parts: &Vec<String>) -> Option<&String> {
    for part in parts.iter().skip(1) {
        if Path::new(part).exists() {
            return Some(part);
        }
    }
    None
}

fn auxiliary_match_rule<F>(command: &CrabCommand, fn_get_actual_file: F) -> bool
where
    F: Fn(&Vec<String>) -> Option<&String>,
{
    if let Some(output) = &command.output {
        if command.script_parts.len() > 1 {
            output.contains(": No such file or directory")
                && fn_get_actual_file(&command.script_parts).is_some()
        } else {
            false
        }
    } else {
        false
    }
}

fn mockable_match_rule<F>(
    command: &mut CrabCommand,
    fn_get_actual_file: &F,
    system_shell: Option<&dyn Shell>,
) -> bool
where
    F: Fn(&Vec<String>) -> Option<&String>,
{
    match_rule_with_is_app(
        |command| auxiliary_match_rule(command, fn_get_actual_file),
        command,
        vec!["grep", "egrep"],
        None,
    )
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    mockable_match_rule(command, &get_actual_file, system_shell)
}

fn mockable_get_new_command<F>(
    command: &mut CrabCommand,
    fn_get_actual_file: &F,
    system_shell: Option<&dyn Shell>,
) -> Vec<String>
where
    F: Fn(&Vec<String>) -> Option<&String>,
{
    if command.script_parts.len() > 1 {
        if let Some(actual_file) = fn_get_actual_file(&command.script_parts) {
            let mut parts = command.script_parts.clone();
            parts.remove(parts.iter().position(|x| x == actual_file).unwrap());
            parts.push(actual_file.to_string());
            vec![parts.join(" ")]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}
pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    mockable_get_new_command(command, &get_actual_file, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "grep_arguments_order".to_owned(),
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
    use super::{mockable_get_new_command, mockable_match_rule};
    use crate::cli::command::CrabCommand;
    use rstest::rstest;

    fn mocked_get_actual_file(parts: &Vec<String>) -> Option<&String> {
        for part in parts.iter().skip(1) {
            if !part.starts_with("-") {
                return Some(part);
            }
        }
        None
    }

    #[rstest]
    #[case("grep test.py test", "grep: test: No such file or directory", true)]
    #[case("grep -lir . test", "grep: test: No such file or directory", true)]
    #[case("egrep test.py test", "egrep: test: No such file or directory", true)]
    #[case("egrep -lir . test", "egrep: test: No such file or directory", true)]
    #[case("cat test.py", "cat: test.py: No such file or directory", false)]
    #[case("grep test test.py", "", false)]
    #[case("grep -lir test .", "", false)]
    #[case("egrep test test.py", "", false)]
    #[case("egrep -lir test .", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(
            mockable_match_rule(&mut command, &mocked_get_actual_file, None),
            is_match
        );
    }

    #[rstest]
    #[case("grep test.py test", "grep: test: No such file or directory", vec!["grep test test.py"])]
    #[case("grep -lir . test", "grep: test: No such file or directory", vec!["grep -lir test ."])]
    #[case("grep . test -lir", "grep: test: No such file or directory", vec!["grep -lir . test"])]
    #[case("egrep test.py test", "egrep: test: No such file or directory", vec!["egrep test test.py"])]
    #[case("egrep -lir . test", "egrep: test: No such file or directory", vec!["egrep -lir test ."])]
    #[case("egrep . test -lir", "egrep: test: No such file or directory", vec!["egrep -lir . test"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(mockable_get_new_command(&mut command, &mocked_get_actual_file, None), expected);
    }
}
