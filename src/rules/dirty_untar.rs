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
    cmd_split.len() > 1 && cmd_split[1].contains('x')
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
    !command.script.contains("-C")
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
        nonw => return vec![],
    };
}

pub fn side_effect(old_cmd: CrabCommand, command: Option<&str>) {
    if let Some((filepath, _)) = tar_file(&old_cmd.script_parts) {
        let mut archive = Archive::new(std::fs::File::open(filepath).unwrap());

        for file in archive.entries().unwrap() {
            let file = file.unwrap();
            let path = file.path().unwrap().to_path_buf();

            let filename = path.to_string_lossy();
            if !filename.starts_with("._") {
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

#[cfg(test)]
mod tests {
    use super::{get_new_command, match_rule, side_effect, TAR_EXTENSIONS};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;
    use tar::Archive;
    use tar::Builder;
    use tempfile::TempDir;

    pub fn tar_error(filename: &str, tmp_dir: &TempDir) {
        let filename = format!("./{}", filename);
        let path = tmp_dir.path().join(&filename);

        let _ = env::set_current_dir(tmp_dir.path());
        reset(&path);

        let entries = fs::read_dir(".").unwrap();
        let mut files = entries
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .unwrap();
        let mut expected_files = vec![
            Path::new(&filename),
            Path::new("./a"),
            Path::new("./b"),
            Path::new("./c"),
            Path::new("./d"),
        ];
        expected_files.sort();
        files.sort();
        assert_eq!(files, expected_files);

        let entries = fs::read_dir("./d").unwrap();
        let mut files = entries
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .unwrap();
        files.sort();
        assert_eq!(files, vec![Path::new("./d/e")]);
    }

    fn reset(path: &Path) {
        fs::create_dir("d").unwrap();
        let files = vec!["a", "b", "c", "d/e"];

        let tar_gz = File::create(path).unwrap();
        let mut tar = Builder::new(tar_gz);

        for file in files {
            let file_path = PathBuf::from(file);

            let mut f = File::create(&file_path).unwrap();
            f.write_all(b"*").unwrap();
            tar.append_path(&file_path).unwrap();
            fs::remove_file(&file_path).unwrap();
        }

        let tar_gz = File::open(path).unwrap();
        let mut tar = Archive::new(tar_gz);
        tar.unpack(".").unwrap();
    }

    fn get_filename() -> Vec<(
        Box<dyn Fn(&str) -> String>,
        Box<dyn Fn(&str) -> String>,
        Box<dyn Fn(&str) -> String>,
    )> {
        vec![
            (
                Box::new(|s: &str| format!("foo{}", s)),
                Box::new(|s: &str| format!("foo{}", s)),
                Box::new(|s: &str| format!("foo{}", s)),
            ),
            (
                Box::new(|s: &str| format!(r#""foo bar{}""#, s)),
                Box::new(|s: &str| format!("foo bar{}", s)),
                Box::new(|s: &str| format!("'foo bar{}'", s)),
            ),
        ]
    }

    fn get_script() -> Vec<(
        Box<dyn Fn(&str) -> String>,
        Box<dyn Fn(&str, &str) -> String>,
    )> {
        vec![
            (
                Box::new(|s: &str| format!("tar xvf {}", s)),
                Box::new(|dir: &str, filename: &str| {
                    format!(
                        "mkdir -p {dir} && tar xvf {filename} -C {dir}",
                        dir = dir,
                        filename = filename
                    )
                }),
            ),
            (
                Box::new(|s: &str| format!("tar -xvf {}", s)),
                Box::new(|dir: &str, filename: &str| {
                    format!(
                        "mkdir -p {dir} && tar -xvf {filename} -C {dir}",
                        dir = dir,
                        filename = filename
                    )
                }),
            ),
            (
                Box::new(|s: &str| format!("tar --extract -f {}", s)),
                Box::new(|dir: &str, filename: &str| {
                    format!(
                        "mkdir -p {dir} && tar --extract -f {filename} -C {dir}",
                        dir = dir,
                        filename = filename
                    )
                }),
            ),
        ]
    }

    #[test]
    // The unit tests were split into test_match, test_side_effect and test_get_new_command.
    // However, there was an issue with tempfile raising errors when the tests were running in
    // parallel. Hence, we moved them to the same function.
    fn test_dirty_unrar() {
        for (filename, unquoted, quoted) in get_filename() {
            for (script, fixed) in get_script() {
                for ext in TAR_EXTENSIONS {
                    let tmp_dir = TempDir::new().unwrap();
                    tar_error(&unquoted(ext), &tmp_dir);
                    let mut command =
                        CrabCommand::new(script(&filename(ext)), Some("".to_owned()), None);
                    assert!(match_rule(&mut command, None));

                    side_effect(command, None);
                    let entries = fs::read_dir(".").unwrap();
                    let mut files = entries
                        .map(|res| res.map(|e| e.path()))
                        .collect::<Result<Vec<_>, std::io::Error>>()
                        .unwrap();
                    let unquoted = format!("./{}", unquoted(ext));
                    let mut expected_files = vec![Path::new(&unquoted), Path::new("./d")];
                    files.sort();
                    expected_files.sort();
                    assert_eq!(files, expected_files);

                    let system_shell = Bash {};
                    let mut command =
                        CrabCommand::new(script(&filename(ext)), Some("".to_owned()), None);
                    assert_eq!(
                        get_new_command(&mut command, Some(&system_shell)),
                        vec![fixed(&quoted(""), &filename(ext))]
                    );
                }
            }
        }
    }
}
