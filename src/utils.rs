use similar::get_close_matches as difflib_get_close_matches;
use std::collections::HashSet;
use std::env;

use std::path::Path;

use crate::cli::command::CrabCommand;
use crate::shell::Shell;

use regex::Regex;

pub fn replace_argument(script: &str, from_: &str, to: &str) -> String {
    let re = Regex::new(&format!(" {}$", regex::escape(from_))).unwrap();
    let replaced_in_the_end = re.replace(script, &format!(" {}", to));

    if replaced_in_the_end != script {
        replaced_in_the_end.into_owned()
    } else {
        script.replace(&format!(" {} ", from_), &format!(" {} ", to))
    }
}

pub fn replace_command(command: &CrabCommand, broken: &str, matched: Vec<&str>) -> Vec<String> {
    let candidate_commands = get_close_matches(broken, &matched, Some(0.1));
    let mut new_commands = Vec::<String>::new();
    for cmd in candidate_commands {
        new_commands.push(replace_argument(&command.script, broken, cmd.trim()));
    }
    new_commands
}

pub fn get_closest<'a>(
    word: &'a str,
    possibilities: &'a [&'a str],
    cutoff: Option<f32>,
    fallback_to_first: bool,
) -> &'a str {
    let cutoff = cutoff.unwrap_or(0.6);
    let matches = difflib_get_close_matches(word, possibilities, 1, cutoff);
    if matches.len() == 0 && fallback_to_first{
        possibilities[0]
    }else {
        matches[0]
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
    cutoff: Option<f32>,
) -> Vec<&'a str> {
    // TODO: Read parameters from config file
    let n = 3;
    let cutoff = cutoff.unwrap_or(0.6);
    difflib_get_close_matches(word, possibilities, n, cutoff)
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

#[cfg(test)]
mod tests {
    use mockall::mock;

    use crate::{cli::command::CrabCommand, shell::Shell, utils::get_alias};

    use super::get_valid_history_without_current;

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
}
