use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        (command
            .script_parts
            .get(1)
            .is_some_and(|s| s == "uninstall" || s == "rm" || s == "remove"))
            && output.contains("brew uninstall --force")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["brew"], Some(2))
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let mut command_parts = command.script_parts.clone();
    "uninstall".clone_into(&mut command_parts[1]);
    command_parts.insert(2, "--force".to_owned());
    vec![command_parts.join(" ")]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "brew_uninstall".to_owned(),
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
    use rstest::rstest;

    const OUTPUT: &str = "Uninstalling /usr/local/Cellar/tbb/4.4-20160916... (118 files, 1.9M)\ntbb 4.4-20160526, 4.4-20160722 are still installed.\nRemove all versions with `brew uninstall --force tbb`.\n";

    #[rstest]
    #[case("brew uninstall tbb", OUTPUT, true)]
    #[case("brew rm tbb", OUTPUT, true)]
    #[case("brew remove tbb", OUTPUT, true)]
    #[case(
        "brew remove gnuplot",
        "Uninstalling /usr/local/Cellar/gnuplot/5.0.4_1... (44 files, 2.3M)\n",
        false
    )]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("brew uninstall tbb", OUTPUT, vec!["brew uninstall --force tbb"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
