use fuzzt::{algorithms::SequenceMatcher, get_top_n};
use std::collections::HashSet;
use std::env;

use std::path::Path;

use crate::cli::command::CrabCommand;
use crate::shell::Shell;

use regex::Regex;

/// This function prints a message to the console when the program is
/// compiled in debug mode and does nothing in release mode.
///
/// # Arguments
///
/// * `message` - A string slice that holds the message to be logged.
///
/// # Examples
///
/// ```
/// debug_log("This is a debug log message.");
/// ```
#[cfg(debug_assertions)]
pub fn debug_log(message: &str) {
    println!("{}", message);
}

#[cfg(not(debug_assertions))]
pub fn debug_log(_: &str) {
    // This function does nothing when not in debug mode.
}

/// Replaces an argument in a script.
///
/// This function takes a script and two strings `from_` and `to`. It replaces the last occurrence of `from_` in the script with `to`.
/// If `from_` does not occur at the end of the script, it replaces all occurrences of `from_` in the script with `to`.
///
/// # Arguments
///
/// * `script` - A string slice that holds the script.
/// * `from_` - The string to be replaced.
/// * `to` - The string to replace with.
///
/// # Returns
///
/// This function returns a new string with the replaced argument.
pub fn replace_argument(script: &str, from_: &str, to: &str) -> String {
    let re = Regex::new(&format!(" {}$", regex::escape(from_))).unwrap();
    let replaced_in_the_end = re.replace(script, &format!(" {}", to));

    if replaced_in_the_end != script {
        replaced_in_the_end.into_owned()
    } else {
        script.replace(&format!(" {} ", from_), &format!(" {} ", to))
    }
}

/// Replaces a broken command with close matches from a list of candidate commands.
///
/// Given a `CrabCommand` and a `broken` command string, this function attempts to find close matches
/// within a provided list of `matched` candidate strings. It then replaces the `broken` command in the
/// script of the `CrabCommand` with each of the close matches, trimming any leading or trailing whitespace.
///
/// # Arguments
///
/// * `command` - A reference to a `CrabCommand` that holds the script where the replacement should occur.
/// * `broken` - The string representing the broken command to be replaced.
/// * `matched` - A vector of string slices representing the candidate commands for replacement.
///
/// # Returns
///
/// This function returns a vector of strings, each containing the script of the `CrabCommand` with the
/// `broken` command replaced by one of the close matches.
///
/// # Examples
///
/// ```
/// let command = CrabCommand { script: "echo broken_command" };
/// let broken = "broken_command";
/// let matched = vec!["fixed_command1", "fixed_command2"];
/// let fixed_scripts = replace_command(&command, broken, matched);
/// assert_eq!(fixed_scripts, vec!["echo fixed_command1", "echo fixed_command2"]);
/// ```
pub fn replace_command(command: &CrabCommand, broken: &str, matched: Vec<&str>) -> Vec<String> {
    let candidate_commands = get_close_matches(broken, &matched, None, Some(0.2));
    let mut new_commands = Vec::<String>::new();
    for cmd in candidate_commands {
        new_commands.push(replace_argument(&command.script, broken, cmd.trim()));
    }
    new_commands
}

/// Returns the closest match for a given word from a list of possibilities.
///
/// # Arguments
///
/// * `word` - A string slice that holds the word for which to find the closest match.
/// * `possibilities` - A slice of string slices that holds the list of words to check against.
/// * `cutoff` - An Option that holds the cutoff similarity ratio. The function returns the closest match that has a similarity ratio greater than or equal to the cutoff. If the cutoff is None, it defaults to 0.6.
/// * `fallback_to_first` - A boolean that indicates whether to return the first word from the list of possibilities if no match is found that meets the cutoff.
///
/// # Returns
///
/// * An Option that contains a string slice. If a match is found, it contains the closest match. If no match is found, it contains None, unless `fallback_to_first` is true, in which case it contains the first word from the list of possibilities.
pub fn get_closest<'a>(
    word: &'a str,
    possibilities: &'a [&'a str],
    cutoff: Option<f64>,
    fallback_to_first: Option<bool>,
) -> Option<&'a str> {
    let cutoff = cutoff.unwrap_or(0.6);
    let fallback_to_first = fallback_to_first.unwrap_or(true);
    let matches = get_top_n(
        word,
        possibilities,
        Some(cutoff),
        Some(1),
        None,
        Some(&SequenceMatcher),
    );
    if matches.is_empty() {
        if fallback_to_first {
            Some(possibilities[0])
        } else {
            None
        }
    } else {
        Some(matches[0])
    }
}

/// Gets a list of close matches for a word from a list of possibilities.
///
/// # Arguments
///
/// * `word` - The word to find matches for.
/// * `possibilities` - A slice of strings representing the possibilities.
///
/// # Returns
///
/// A vector of close matches for the given word.
pub fn get_close_matches<'a>(
    word: &'a str,
    possibilities: &'a [&'a str],
    n_matches: Option<usize>,
    cutoff: Option<f64>,
) -> Vec<&'a str> {
    // TODO: Read parameters from config file
    let n = n_matches.unwrap_or(3);
    let cutoff = cutoff.unwrap_or(0.6);
    get_top_n(
        word,
        possibilities,
        Some(cutoff),
        Some(n),
        None,
        Some(&SequenceMatcher),
    )
}

/// Gets the alias for the OC_ALIAS environment variable or defaults to "crab".
///
/// # Returns
///
/// The alias as a String.
pub fn get_alias() -> String {
    env::var("OC_ALIAS").unwrap_or("crab".to_owned())
}

/// Gets a vector of all executables in the PATH excluding certain entry points.
///
/// # Returns
///
/// A vector of executable names.
pub fn get_all_executable() -> Vec<String> {
    let tf_entry_points = ["ohcrab", "crab"];

    let mut bins = vec![];

    if let Ok(path_var) = env::var("PATH") {
        for path in env::split_paths(&path_var) {
            if let Ok(iterdir) = Path::new(&path).read_dir() {
                for executable in iterdir.flatten() {
                    if let Ok(file_type) = executable.file_type() {
                        if !file_type.is_dir() {
                            if let Some(file_name) = executable.path().file_name() {
                                if let Some(name) = file_name.to_str() {
                                    if !tf_entry_points.contains(&name) {
                                        bins.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // TODO: Add shell aliases
    bins
}

/// Filters out history entries occurring immediately after the alias ("crab").
///
/// # Arguments
///
/// * `history` - A reference to a vector of strings representing the history.
/// * `oc_alias` - A reference to a string representing the alias to be filtered out.
///
/// # Returns
///
/// * Vector of strings where entries immediately after the alias have been filtered out.
fn not_corrected(history: &Vec<String>, oc_alias: &String) -> Vec<String> {
    let mut previous: Option<&str> = None;
    let mut result = Vec::new();

    for line in history {
        if let Some(prev) = previous {
            if line != oc_alias {
                result.push(prev.to_string());
            }
        }
        previous = Some(line);
    }

    if let Some(last) = history.last() {
        result.push(last.clone());
    }
    result
}

/// Returns a vector of valid history commands excluding the current command.
///
/// The method compares the command with executables and shell builtins and
/// ignores commands performed just after the alias is called ("crab").
///
/// # Arguments
///
/// * `command` - A reference to the current CrabCommand.
/// * `system_shell` - A reference to the system shell.
///
/// # Returns
///
/// * A vector of valid history commands as strings.
pub fn get_valid_history_without_current(
    command: &CrabCommand,
    system_shell: &dyn Shell,
) -> Vec<String> {
    let mut corrected: Vec<String> = Vec::new();
    let mut valid_history: Vec<String> = Vec::new();

    let history = system_shell.get_history(None);
    let mut executables = history.clone();
    executables.extend(system_shell.get_builtin_commands());
    executables.extend(get_all_executable());
    let executables: HashSet<_> = executables.into_iter().collect();

    for line in not_corrected(&history, &get_alias()) {
        let first_word = line.split_whitespace().next().unwrap_or(line.as_str());
        if !line.starts_with(&get_alias())
            & (line != command.script)
            & executables.contains(first_word)
        {
            valid_history.push(line);
        }
    }

    valid_history
}

/// Returns a vector of matched commands from the given stderr string.
///
/// This function iterates over each line in `stderr`. If a line contains any of the separators,
/// it sets a flag `should_yield` to true. For each subsequent line, if `should_yield` is true and
/// the line is not empty, the function adds the line to the vector of matched commands.
///
/// # Arguments
///
/// * `stderr` - A string slice that holds the standard error output.
/// * `separator_option` - An Option that contains a vector of separator strings. If this option is None,
///   the function uses ["Did you mean"] as the default separator.
///
/// # Returns
///
/// This function returns a vector of matched commands. Each command is a string that follows a line
/// containing a separator and does not contain a separator itself.
///
/// # Example
///
/// ```
/// let stderr = "error: pathspec 'feature/test_commit' did not match any file(s) known to git\nDid you mean this?\n    origin/feature/test_commit";
/// let commands = get_all_matched_commands(stderr, None);
/// assert_eq!(commands, vec!["origin/feature/test_commit"]);
/// ```
pub fn get_all_matched_commands(stderr: &str, separator_option: Option<Vec<&str>>) -> Vec<String> {
    let separator = match separator_option {
        None => vec!["Did you mean"],
        Some(sep) => sep,
    };
    let mut should_yield = false;
    let mut matched_commands = Vec::new();

    for line in stderr.lines() {
        if separator.iter().any(|&sep| line.contains(sep)) {
            should_yield = true;
        } else if should_yield && !line.is_empty() {
            matched_commands.push(line.trim().to_string());
        }
    }

    matched_commands
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use crate::{cli::command::CrabCommand, shell::Shell, utils::get_alias};

    use super::get_all_matched_commands;
    use super::get_valid_history_without_current;
    use rstest::rstest;

    mock! {
        pub MyShell {}
        impl Shell for MyShell {
            fn app_alias(&self, alias_name: &str) -> String;
            fn get_shell(&self) -> String;
            fn get_history_file_name(&self) -> String;
            fn script_from_history(&self, command_script: &str) -> String;
            fn get_history<'a>(&self, file_path: Option<&'a str>) -> Vec<String> ;
            fn get_builtin_commands(&self) -> Vec<String>;
        }
    }

    #[test]
    fn test_get_valid_history_without_current() {
        let command =
            CrabCommand::new("ls -l".to_owned(), Some("multiple\nlines".to_owned()), None);
        let mut mock_shell = MockMyShell::new();
        mock_shell
            .expect_get_builtin_commands()
            .returning(|| vec!["command1".to_string(), "command2".to_string()]);
        mock_shell.expect_get_history().returning(|_| {
            vec![
                "ls -l".to_string(),
                "command1".to_string(),
                "cmp a.txt b.txt".to_string(),
            ]
        });
        let system_shell: Box<dyn Shell> = Box::new(mock_shell);

        assert_eq!(
            vec!["command1", "cmp a.txt b.txt"],
            get_valid_history_without_current(&command, &*system_shell)
        );

        let mut mock_shell = MockMyShell::new();
        mock_shell
            .expect_get_builtin_commands()
            .returning(|| vec!["command1".to_string(), "command2".to_string()]);
        mock_shell.expect_get_history().returning(|_| {
            vec![
                "ls -l".to_string(),
                "cmp a.txt b.txt".to_string(),
                get_alias(),
            ]
        });
        let system_shell: Box<dyn Shell> = Box::new(mock_shell);
        // Skip "cmp a.txt b.txt" because it comes before "crab" (alias)
        assert_eq!(
            Vec::<String>::new(),
            get_valid_history_without_current(&command, &*system_shell)
        );
    }

    #[rstest]
    #[case("git: 'cone' is not a git command. See 'git --help'.\n\nDid you mean one of these?\n\tclone", vec!["clone"])]
    #[case("git: 're' is not a git command. See 'git --help'.\n\nDid you mean one of these?\n\trebase\n\treset\n\tgrep\n\trm", vec!["rebase", "reset", "grep", "rm"])]
    #[case("tsuru: \"target\" is not a tsuru command. See \"tsuru help\".\n\nDid you mean one of these?\n\tservice-add\n\tservice-bind\n\tservice-doc\n\tservice-info\n\tservice-list\n\tservice-remove\n\tservice-status\n\tservice-unbind", vec![ "service-add", "service-bind", "service-doc", "service-info", "service-list", "service-remove", "service-status", "service-unbind"])]
    fn test_get_all_matched_commands(#[case] stderr: &str, #[case] result: Vec<&str>) {
        assert_eq!(get_all_matched_commands(stderr, None), result);
    }
}
