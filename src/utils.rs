use similar::get_close_matches as difflib_get_close_matches;
use std::collections::HashSet;
use std::env;
use std::iter::FromIterator;
use std::path::Path;

use crate::cli::command::CrabCommand;
use crate::shell::Shell;

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
pub fn get_close_matches<'a>(word: &'a str, possibilities: &'a [&'a str]) -> Vec<&'a str> {
    // TODO: Read parameters from config file
    let n = 3;
    let cutoff = 0.6;
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

pub fn get_valid_history_without_current(
    command: &CrabCommand,
    system_shell: &Box<dyn Shell>,
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

// def get_valid_history_without_current(command):
//     def _not_corrected(history, tf_alias):
//         """Returns all lines from history except that comes before `fuck`."""
//         previous = None
//         for line in history:
//             if previous is not None and line != tf_alias:
//                 yield previous
//             previous = line
//         if history:
//             yield history[-1]
//     from thefuck.shells import shell
//     history = shell.get_history()
//     tf_alias = get_alias()
//     executables = set(get_all_executables())\
//         .union(shell.get_builtin_commands())
//     return [line for line in _not_corrected(history, tf_alias)
//             if not line.startswith(tf_alias) and not line == command.script
//             and line.split(' ')[0] in executables]

#[cfg(test)]
mod tests {
    use mockall::{mock, predicate};

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
        let command = CrabCommand::new("ls -l".to_owned(), Some("multiple\nlines".to_owned()), None);
        // let mock_shell: Box<dyn Shell> = Box::new(MockShell {});
        let mut mock_shell = MockMyShell::new();
        mock_shell
            .expect_get_builtin_commands()
            .returning(|| vec!["command1".to_string(), "command2".to_string()]);
        mock_shell
            .expect_get_history()
            .returning(|_| {
                vec![
                    "ls -l".to_string(),
                    "command1".to_string(),
                    "cmp a.txt b.txt".to_string(),
                ]
            });
        let system_shell: Box<dyn Shell> = Box::new(mock_shell);

        assert_eq!(
            vec!["command1", "cmp a.txt b.txt"],
            get_valid_history_without_current(&command, &system_shell)
        );

        let mut mock_shell = MockMyShell::new();
        mock_shell
            .expect_get_builtin_commands()
            .returning(|| vec!["command1".to_string(), "command2".to_string()]);
        mock_shell
            .expect_get_history()
            .returning(|_| {
                vec![
                    "ls -l".to_string(),
                    "cmp a.txt b.txt".to_string(),
                    get_alias()
                ]
            });
        let system_shell: Box<dyn Shell> = Box::new(mock_shell);
        // Skip "cmp a.txt b.txt" because it comes before "crab" (alias)
        assert_eq!(
            Vec::<String>::new(),
            get_valid_history_without_current(&command, &system_shell)
        );
    }
}
