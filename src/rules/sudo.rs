use super::{
    get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule,
};
use crate::{cli::command::CrabCommand, shell::Shell};

const PATTERNS: [&str; 28] = [
    "permission denied",
    "eacces",
    "pkg: insufficient privileges",
    "you cannot perform this operation unless you are root",
    "non-root users cannot",
    "operation not permitted",
    "not super-user",
    "superuser privilege",
    "root privilege",
    "this command has to be run under the root user.",
    "this operation requires root.",
    "requested operation requires superuser privilege",
    "must be run as root",
    "must run as root",
    "must be superuser",
    "must be root",
    "need to be root",
    "need root",
    "needs to be run as root",
    "only root can ",
    "you don't have access to the history db.",
    "authentication is required",
    "edspermissionerror",
    "you don't have write permissions",
    "use `sudo`",
    "sudorequirederror",
    "error: insufficient privileges",
    "updatedb: can not open a temporary file",
];

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script_parts.first().map_or(false, |s| {
            s != "sudo" && PATTERNS.iter().any(|&pattern| output.contains(pattern))
        })
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| {
            match_rule_with_is_app(
                auxiliary_match_rule,
                command,
                vec!["ls", "echo", "mkdir"],
                None,
            )
        },
        command,
    )
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    if command.script.contains("&&") {
        vec![format!("sudo sh -c \"{}\"", command.script_parts.join(" "))]
    } else if command.script.contains(">") {
        vec![format!(
            "sudo sh -c \"{}\"",
            command.script.replace("\"", "\\\"")
        )]
    } else {
        vec![format!("sudo {}", command.script)]
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "sudo".to_owned(),
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
    #[case("", "Permission denied", false)]
    #[case("sudo ls", "Permission denied", false)]
    #[case("", "", false)]
    #[case("ls", "Permission denied", true)]
    #[case("echo a > b", "Permission denied", true)]
    #[case("echo \"a\" >> b", "Permission denied", true)]
    #[case("mkdir && touch a", "Permission denied", true)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("ls", "Permission denied", vec!["sudo ls"])]
    #[case("echo a > b", "Permission denied", vec!["sudo sh -c \"echo a > b\""])]
    #[case("echo \"a\" >> b", "Permission denied", vec!["sudo sh -c \"echo \\\"a\\\" >> b\""])]
    #[case("mkdir && touch a", "Permission denied", vec!["sudo sh -c \"mkdir && touch a\""])]
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
