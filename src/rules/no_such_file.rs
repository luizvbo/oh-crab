use super::{
    get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule,
};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn get_file(command_output: &str) -> Option<String> {
    let patterns = [
        r"mv: cannot move '[^']*' to '([^']*)': No such file or directory",
        r"mv: cannot move '[^']*' to '([^']*)': Not a directory",
        r"cp: cannot create regular file '([^']*)': No such file or directory",
        r"cp: cannot create regular file '([^']*)': Not a directory",
    ];
    for pattern in patterns.iter() {
        let re = Regex::new(pattern).unwrap();
        if let Some(caps) = re.captures(command_output) {
            return caps.get(1).map(|m| m.as_str().to_owned());
        }
    }
    None
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command
            .script_parts
            .first()
            .is_some_and(|s| s == "mv" || s == "cp")
            && get_file(output).is_some()
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["mv", "cp"], None),
        command,
    )
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    if let Some(output) = &command.output {
        let file = get_file(output);
        if let Some(file) = file {
            let dir = &file[0..file.rfind('/').unwrap_or(0)];
            vec![format!("mkdir -p {} && {}", dir, command.script)]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "no_such_file".to_owned(),
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
        "mv foo bar/foo",
        "mv: cannot move 'foo' to 'bar/foo': No such file or directory",
        true
    )]
    #[case(
        "mv foo bar/",
        "mv: cannot move 'foo' to 'bar/': No such file or directory",
        true
    )]
    #[case(
        "cp foo bar/",
        "cp: cannot create regular file 'bar/': Not a directory",
        true
    )]
    #[case("mv foo bar/", "", false)]
    #[case("mv foo bar/foo", "mv: permission denied", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("mv foo bar/foo", "mv: cannot move 'foo' to 'bar/foo': No such file or directory", vec!["mkdir -p bar && mv foo bar/foo"])]
    #[case("mv foo bar/", "mv: cannot move 'foo' to 'bar/': No such file or directory", vec!["mkdir -p bar && mv foo bar/"])]
    #[case("cp foo bar/", "cp: cannot create regular file 'bar/': Not a directory", vec!["mkdir -p bar && cp foo bar/"])]
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
