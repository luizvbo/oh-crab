use crate::cli::command::CrabCommand;
use std::path::Path;

pub mod git;
pub mod parameterized_tests;

/// Matches a rule with a given command if it is an application.
///
/// # Arguments
///
/// * `func` - Match function that takes a `CrabCommand` and returns a boolean.
/// * `command` - A reference to a `CrabCommand` instance.
/// * `app_names` - A vector of application names to check against.
/// * `at_least` - An optional usize that specifies the minimum number of script parts.
///
/// # Returns
///
/// * `bool` - Returns true if the command matches the rule and is an application, false otherwise.
pub fn match_rule_with_is_app<F>(
    func: F,
    command: &CrabCommand,
    app_names: Vec<&str>,
    at_least: Option<usize>,
) -> bool
where
    F: Fn(&CrabCommand) -> bool,
{
    if is_app(command, app_names, at_least) {
        func(command)
    } else {
        false
    }
}

/// Checks if a given command is an application.
///
/// # Arguments
///
/// * `command` - A reference to a `CrabCommand` instance.
/// * `app_names` - A vector of application names to check against.
/// * `at_least` - An optional usize that specifies the minimum number of script parts.
///
/// # Returns
///
/// * `bool` - Returns true if the command is an application, false otherwise.
fn is_app(command: &CrabCommand, app_names: Vec<&str>, at_least: Option<usize>) -> bool {
    let at_least = at_least.unwrap_or(0);
    if command.script_parts.len() > at_least {
        let app_name = Path::new(&command.script_parts[0])
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("");
        return app_names.contains(&app_name);
    }

    false
}
