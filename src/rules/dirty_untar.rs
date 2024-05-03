use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use shlex::Quoter;
use std::fs;
use tar::Archive;

const TAR_EXTENSIONS: [&str; 15] = [
    ".tar",
    ".tar.Z",
    ".tar.bz2",
    ".tar.gz",
    ".tar.lz",
    ".tar.lzma",
    ".tar.xz",
    ".taz",
    ".tb2",
    ".tbz",
    ".tbz2",
    ".tgz",
    ".tlz",
    ".txz",
    ".tz",
];

fn is_tar_extract(cmd: &str) -> bool {
    if cmd.contains("--extract") {
        return true;
    }
    let cmd_split: Vec<&str> = cmd.split_whitespace().collect();
    return cmd_split.len() > 1 && cmd_split[1].contains('x');
}

fn tar_file(cmd: &[String]) -> Option<(String, String)> {
    for c in cmd {
        for ext in &TAR_EXTENSIONS {
            if c.ends_with(ext) {
                return Some((c.clone(), c[..c.len() - ext.len()].to_string()));
            }
        }
    }
    None
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    command.script.contains("-C")
        && is_tar_extract(&command.script)
        && tar_file(&command.script_parts).is_some()
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["tar"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let shlex_quoter = Quoter::new();
    return match tar_file(&command.script_parts) {
        Some((_, filepath_no_ext)) => {
            let dir = shlex_quoter.quote(&filepath_no_ext).unwrap();
            vec![system_shell.unwrap().and(vec![
                &format!("mkdir -p {}", dir),
                &format!("{cmd} -C {dir}", dir = dir, cmd = command.script),
            ])]
        }
        Nonw => return vec![],
    };
}

pub fn side_effect(old_cmd: CrabCommand, command: &String) {
    if let Some((filepath, _)) = tar_file(&old_cmd.script_parts) {
        let archive = Archive::new(std::fs::File::open(filepath).unwrap());

        for file in archive.entries().unwrap() {
            let file = file.unwrap();
            let path = file.path().unwrap().to_path_buf();

            if !path
                .canonicalize()
                .unwrap()
                .starts_with(std::env::current_dir().unwrap())
            {
                // it's unsafe to overwrite files outside of the current directory
                continue;
            }

            if path.is_file() {
                fs::remove_file(path).unwrap_or(());
            }
        }
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "tar".to_owned(),
        None,
        None,
        None,
        match_rule,
        get_new_command,
        Some(side_effect),
    )
}
