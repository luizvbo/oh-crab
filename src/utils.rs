use similar::get_close_matches as difflib_get_close_matches;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::env;
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

fn not_corrected(history: &Vec<String>, oc_alias: &String) -> Vec<String>{
    let mut previous: Option<String> = None;
    let mut result = Vec::new();

    for line in history {
        if let Some(prev) = &previous {
            if line != oc_alias {
                result.push(prev.clone());
            }
        }
        previous = Some(line.clone());
    }

    if let Some(last) = history.last() {
        result.push(last.clone());
    }

    result
}


fn get_valid_history_without_current(
command: &CrabCommand, system_shell: &Box<dyn Shell>) -> Vec<String>{
    let mut corrected: Vec<String> = Vec::new();
    let mut valid_history: Vec<String> = Vec::new();

    let mut executables = system_shell.get_history();
    executables.extend(system_shell.get_builtin_commands());
    let executables: HashSet<_> = executables.into_iter().collect();
        
    for line in not_corrected(&system_shell.get_history(), &get_alias()){
        if !line.starts_with(get_alias()) & line != command.script & line.split(" ")[0] in executables{
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
//

