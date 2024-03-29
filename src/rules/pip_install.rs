use super::{match_rule_without_sudo, utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script.contains("pip install") && output.contains("Permission denied")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["pip"], None),
        command,
    )
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if !command.script.contains("--user") {
        vec![command.script.replace(" install ", " install --user ")]
    } else {
        vec![format!("sudo {}", command.script.replace(" --user", ""))]
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "pip_install".to_owned(),
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

    const ERROR_PERMISSION_DENIED: &str = "\nCould not install packages due to an EnvironmentError: [Errno 13] Permission denied: '/Library/Python/2.7/site-packages/entrypoints.pyc'\nConsider using the `--user` option or check the permissions.\n";
    const SUCCESS_INSTALL: &str = "\nCollecting bacon\n  Downloading https://files.pythonhosted.org/packages/b2/81/19fb79139ee71c8bc4e5a444546f318e2b87253b8939ec8a7e10d63b7341/bacon-0.3.1.zip (11.0MB)\n    100% |████████████████████████████████| 11.0MB 3.0MB/s\nInstalling collected packages: bacon\n  Running setup.py install for bacon ... done\nSuccessfully installed bacon-0.3.1\n";

    #[rstest]
    #[case("pip install -r requirements.txt", ERROR_PERMISSION_DENIED, true)]
    #[case("pip install bacon", SUCCESS_INSTALL, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("pip install -r requirements.txt", "", vec!["pip install --user -r requirements.txt"])]
    #[case("pip install bacon", "", vec!["pip install --user bacon"])]
    #[case("pip install --user -r requirements.txt", "", vec!["sudo pip install -r requirements.txt"])]
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
