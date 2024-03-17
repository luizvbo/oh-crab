use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        (command
            .script_parts
            .get(1)
            .map_or(false, |s| s == "ln" || s == "link"))
            && output.contains("brew link --overwrite --dry-run")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["brew"], Some(2))
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let mut command_parts = command.script_parts.clone();
    command_parts[1] = "link".to_owned();
    command_parts.insert(2, "--overwrite".to_owned());
    command_parts.insert(3, "--dry-run".to_owned());
    vec![command_parts.join(" ")]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "brew_link".to_owned(),
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

    const OUTPUT: &str = "Error: Could not symlink bin/gcp\nTarget /usr/local/bin/gcp\nalready exists. You may want to remove it:\n  rm '/usr/local/bin/gcp'\n\nTo force the link and overwrite all conflicting files:\n  brew link --overwrite coreutils\n\nTo list all files that would be deleted:\n  brew link --overwrite --dry-run coreutils\n";

    #[rstest]
    #[case("brew link coreutils", OUTPUT, true)]
    #[case("brew ln coreutils", OUTPUT, true)]
    #[case("brew link coreutils", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("brew link coreutils", OUTPUT, vec!["brew link --overwrite --dry-run coreutils"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
