use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;
use std::path::Path;

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

/// Provides git support for a given function.
///
/// # Arguments
///
/// * `func` - A function that takes a `CrabCommand` instance and returns a boolean.
/// * `command` - A mutable `CrabCommand` instance.
///
/// # Returns
///
/// * `bool` - Returns the result of the function `func`.
pub fn match_rule_with_git_support<F>(func: F, command: &mut CrabCommand) -> bool
where
    F: Fn(&CrabCommand) -> bool,
{
    // supports GitHub's `hub` command
    // which is recommended to be used with `alias git=hub`
    // but at this point, shell aliases have already been resolved
    if !is_app(command, vec!["git", "hub"], None) {
        return false;
    }

    // perform git aliases expansion
    if let Some(stdout) = &command.stdout {
        if stdout.contains("trace: alias expansion:") {
            let re = Regex::new(r"trace: alias expansion: ([^ ]*) => ([^\n]*)").unwrap();
            if let Some(search) = re.captures(stdout) {
                let alias = search.get(1).map_or("", |m| m.as_str());

                // by default git quotes everything, for example:
                //     'commit' '--amend'
                // which is surprising and does not allow to easily test for
                // eg. 'git commit'
                let expansion = search
                    .get(2)
                    .map_or("", |m| m.as_str())
                    .split_whitespace()
                    .map(|part| format!("\"{}\"", part)) // shell.quote(part)
                    .collect::<Vec<_>>()
                    .join(" ");
                let new_script = command
                    .script
                    .replace(&format!(r"\b{}\b", alias), &expansion);

                command.script = new_script;
            }
        }
    }

    func(command)
}

pub fn get_command_with_git_support<F>(
    func: F,
    command: &mut CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String>
where
    F: Fn(&CrabCommand, Option<&dyn Shell>) -> Vec<String>,
{
    // supports GitHub's `hub` command
    // which is recommended to be used with `alias git=hub`
    // but at this point, shell aliases have already been resolved
    if !is_app(command, vec!["git", "hub"], None) {
        return Vec::<String>::new();
    }

    // perform git aliases expansion
    if let Some(stdout) = &command.stdout {
        if stdout.contains("trace: alias expansion:") {
            let re = Regex::new(r"trace: alias expansion: ([^ ]*) => ([^\n]*)").unwrap();
            if let Some(search) = re.captures(stdout) {
                let alias = search.get(1).map_or("", |m| m.as_str());

                // by default git quotes everything, for example:
                //     'commit' '--amend'
                // which is surprising and does not allow to easily test for
                // eg. 'git commit'
                let expansion = search
                    .get(2)
                    .map_or("", |m| m.as_str())
                    .split_whitespace()
                    .map(|part| format!("\"{}\"", part)) // shell.quote(part)
                    .collect::<Vec<_>>()
                    .join(" ");
                let new_script = command
                    .script
                    .replace(&format!(r"\b{}\b", alias), &expansion);

                command.script = new_script;
            }
        }
    }

    func(command, system_shell)
}
