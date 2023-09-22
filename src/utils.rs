use similar::get_close_matches as difflib_get_close_matches;
use std::env;
use std::path::Path;


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
    env::var("OC_ALIAS").unwrap_or("crab".to_owned()).to_owned()
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
                for executable in iterdir {
                    if let Ok(exe) = executable {
                        if let Ok(file_type) = exe.file_type() {
                            if !file_type.is_dir() {
                                if let Some(file_name) = exe.path().file_name() {
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
    }
    // TODO: Add shell aliases
    bins
}
