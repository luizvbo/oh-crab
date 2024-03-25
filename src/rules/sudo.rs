use super::Rule;
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

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(output) = &command.output {
        if !command.script_parts.is_empty()
            && !command.script_parts.contains(&"&&".to_owned())
            && command.script_parts[0] == "sudo"
        {
            return false;
        }
        for pattern in PATTERNS {
            if output.to_lowercase().contains(pattern) {
                return true;
            }
        }
    }
    false
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if command.script.contains("&&") {
        vec![format!(
            "sudo sh -c \"{}\"",
            command
                .script_parts
                .iter()
                .filter(|s| s != &"sudo")
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )]
    } else if command.script.contains('>') {
        vec![format!(
            "sudo sh -c \"{}\"",
            command.script.replace('"', "\\\"")
        )]
    } else {
        vec![format!("sudo {}", command.script)]
    }
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
    #[case("", "Permission denied", true)]
    #[case("", "permission denied", true)]
    #[case("", "npm ERR! Error: EACCES, unlink", true)]
    #[case("", "requested operation requires superuser privilege", true)]
    #[case("", "need to be root", true)]
    #[case("", "need root", true)]
    #[case("", "shutdown: NOT super-user", true)]
    #[case("", "Error: This command has to be run with superuser privileges (under the root user on most systems).", true)]
    #[case(
        "",
        "updatedb: can not open a temporary file for `/var/lib/mlocate/mlocate.db",
        true
    )]
    #[case("", "must be root", true)]
    #[case("", "You don't have access to the history DB.", true)]
    #[case(
        "",
        "error: [Errno 13] Permission denied: '/usr/local/lib/python2.7/dist-packages/ipaddr.py'",
        true
    )]
    #[case("", "", false)]
    #[case("sudo ls", "Permission denied", false)]
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
