use crate::rules::cd_mkdir::auxiliary_get_new_command;
use crate::utils::get_close_matches;
use crate::{cli::command::CrabCommand, shell::Shell};

use std::env;
use std::fs;
use std::path::{Path, MAIN_SEPARATOR};

use super::{get_new_command_without_sudo, match_rule_without_sudo, Rule};

fn get_sub_dirs(parent: &str) -> Vec<String> {
    let mut sub_dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(parent) {
        let sub_dirs: Vec<String> = entries
            .into_iter()
            .flatten()
            .filter(|entry| entry.metadata().map_or(false, |m| m.is_dir()))
            .map(|entry| entry.path().to_str().unwrap().to_string())
            .collect();
    }
    sub_dirs
}
fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    command.script.starts_with("cd ")
        && (if let Some(output) = &command.output {
            output.to_lowercase().contains("no such file or directory")
                || output.to_lowercase().contains("cd: can\"t cd to")
                || output.to_lowercase().contains("does not exist")
        } else {
            false
        })
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(auxiliary_match_rule, command)
}

fn _get_new_command(command: &CrabCommand) -> Vec<String> {
    if command.script_parts.len() > 1 {
        let mut dest: Vec<&str> = command.script_parts[1].split(MAIN_SEPARATOR).collect();
        let mut cwd;
        if dest.last() == Some(&"") {
            dest.pop();
        }
        if dest[0].is_empty() {
            cwd = MAIN_SEPARATOR.to_string();
            dest.remove(0);
        } else {
            cwd = env::current_dir().unwrap().to_str().unwrap().to_string();
        }

        for directory in dest {
            if directory == "." {
                continue;
            } else if directory == ".." {
                cwd = Path::new(&cwd)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                continue;
            }
            let sub_dirs = get_sub_dirs(&cwd);
            let sub_dirs = sub_dirs.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
            let best_matches = get_close_matches(directory, &sub_dirs, None, None);
            if !best_matches.is_empty() {
                cwd = Path::new(&cwd)
                    .join(best_matches[0])
                    .to_str()
                    .unwrap()
                    .to_string();
            } else {
                return auxiliary_get_new_command(command);
            }
        }
        return vec![format!("cd \"{}\"", cwd)];
    }
    vec![]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cd_correction".to_owned(),
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
    use super::auxiliary_match_rule;
    use crate::cli::command::CrabCommand;
    use rstest::rstest;

    #[rstest]
    #[case("cd foo", "cd: foo: No such file or directory\n", true)]
    #[case("cd foo/bar/baz", "cd: foo: No such file or directory", true)]
    #[case("cd foo/bar/baz", "cd: can\"t cd to foo/bar/baz", true)]
    #[case("cd /foo/bar/", "cd: The directory \"/foo/bar/\" does not exist", true)]
    #[case("cd foo", "", false)]
    #[case("", "", false)]
    fn test_match(#[case] command: &str, #[case] stderr: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), None, Some(stderr.to_owned()));
        assert_eq!(auxiliary_match_rule(&command), is_match);
    }
}
