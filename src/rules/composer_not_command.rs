use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        let output_lower = output.to_lowercase();
        (output_lower.contains("did you mean this?")
            || output_lower.contains("did you mean one of these?"))
            || (command.script_parts.contains(&"install".to_owned())
                && output_lower.contains("composer require"))
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["composer"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let re_broken_cmd = Regex::new(r#"Command "([^']*)" is not defined"#).unwrap();
        let re_new_cmd = Regex::new(r"Did you mean this\?[^\n]*\n\s*([^\n]*)").unwrap();
        let re_new_cmd_alt = Regex::new(r"Did you mean one of these\?[^\n]*\n\s*([^\n]*)").unwrap();
        let broken_cmd;
        let mut new_cmd;
        if command.script_parts.contains(&"install".to_owned())
            && output.to_lowercase().contains("composer require")
        {
            broken_cmd = "install".to_owned();
            new_cmd = "require".to_owned();
        } else {
            if let Some(captures) = re_broken_cmd.captures(&output) {
                broken_cmd = captures.get(1).map_or("", |m| m.as_str()).to_owned();
            } else {
                return Vec::<String>::new();
            }
            if let Some(captures) = re_new_cmd.captures(&output) {
                new_cmd = captures.get(1).map_or("", |m| m.as_str()).to_owned();
            } else if let Some(captures) = re_new_cmd_alt.captures(&output) {
                new_cmd = captures.get(1).map_or("", |m| m.as_str()).to_owned();
            } else {
                return Vec::<String>::new();
            }
        }
        vec![command.script.replace(&broken_cmd, &new_cmd.trim())]
    } else {
        Vec::<String>::new()
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "composer_not_command".to_owned(),
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

    const COMPOSER_NOT_COMMAND: &str = "\n\n                                    \n  [InvalidArgumentException]        \n  Command \"udpate\" is not defined.  \n  Did you mean this?                \n      update                        \n                                    \n\n\n";
    const COMPOSER_NOT_COMMAND_ONE_OF_THIS: &str = "\n\n                                   \n  [InvalidArgumentException]       \n  Command \"pdate\" is not defined.  \n  Did you mean one of these?       \n      selfupdate                   \n      self-update                  \n      update                       \n                                   \n\n\n";
    const COMPOSER_REQUIRE_INSTEAD_OF_INSTALL: &str = "Invalid argument package. Use \"composer require package\" instead to add packages to your composer.json.";

    #[rstest]
    #[case("composer udpate", COMPOSER_NOT_COMMAND, true)]
    #[case("composer pdate", COMPOSER_NOT_COMMAND_ONE_OF_THIS, true)]
    #[case("composer install package", COMPOSER_REQUIRE_INSTEAD_OF_INSTALL, true)]
    #[case("ls update", COMPOSER_NOT_COMMAND, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("composer udpate", COMPOSER_NOT_COMMAND, vec!["composer update"])]
    #[case("composer pdate", COMPOSER_NOT_COMMAND_ONE_OF_THIS, vec!["composer selfupdate"])]
    #[case("composer install package", COMPOSER_REQUIRE_INSTEAD_OF_INSTALL, vec!["composer require package"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
