use crate::{cli::command::CrabCommand, shell::Shell};

use which::which;

use super::Rule;

fn get_executable(command: &CrabCommand) -> &str {
    if command.script_parts[0] == "sudo" {
        &command.script_parts[1]
    } else {
        &command.script_parts[0]
    }
}

fn _match_rule(
    command: &mut CrabCommand,
    system_shell: Option<&dyn Shell>,
    mock_which: Option<bool>,
) -> bool {
    if let Some(stdout) = &command.output {
        if stdout.contains("not found") || stdout.contains("not installed") {
            if let Some(which_return) = mock_which {
                !which_return
            } else {
                which(get_executable(command)).is_err()
            }
        } else {
            false
        }
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    _match_rule(command, system_shell, None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let executable = get_executable(command);
    vec![system_shell.unwrap().and(vec![
        &("sudo apt-get install ".to_owned() + executable),
        &command.script,
    ])]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "apt_get".to_owned(),
        None,
        Some(4000),
        None,
        match_rule,
        get_new_command,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::{_match_rule, get_new_command};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;
    use rstest::rstest;

    #[rstest]
    #[case("vim", "vim: command not found", None, true)]
    #[case("sudo vim", "vim: command not found", None, true)]
    #[case("vim", "The program \"vim\" is currently not installed. You can install it by typing: sudo apt install vim", None, true)]
    #[case("", "", Some(false), false)]
    #[case("vim", "", Some(false), false)]
    #[case("vim", "vim: command not found", Some(true), false)]
    #[case("sudo vim", "vim: command not found", Some(true), false)]
    fn test_match_rule(
        #[case] script: &str,
        #[case] stdout: &str,
        #[case] mock_which: Option<bool>,
        #[case] expected: bool,
    ) {
        let mut command = CrabCommand::new(script.to_owned(), Some(stdout.to_owned()), None);
        // For matching cases, we need to mock `which` to return an error.
        let mock_which_result = if expected { Some(false) } else { mock_which };
        assert_eq!(
            _match_rule(&mut command, None, mock_which_result),
            expected
        );
    }

    #[rstest]
    #[case("vim", "", "sudo apt-get install vim && vim")]
    #[case("git init", "", "sudo apt-get install git && git init")]
    #[case("sudo vim", "", "sudo apt-get install vim && sudo vim")]
    #[case("sudo git init", "", "sudo apt-get install git && sudo git init")]
    fn test_get_new_command(#[case] script: &str, #[case] stdout: &str, #[case] expected: &str) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(script.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(
            get_new_command(&mut command, Some(&system_shell))[0],
            expected
        );
    }
}
