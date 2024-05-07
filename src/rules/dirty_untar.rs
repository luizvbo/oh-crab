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
        let mut archive = Archive::new(std::fs::File::open(filepath).unwrap());

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

#[cfg(test)]
mod tests {
    use super::{get_new_command, match_rule, side_effect};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;
    use rstest::rstest;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tar::Archive;
    use tar::Builder;
    use tempfile::TempDir;

    pub fn tar_error(filename: &str) {
        let tmpdir = TempDir::new().unwrap();
        let path = tmpdir.path().join(filename);

        reset(&path);

        let entries = fs::read_dir(".").unwrap();
        let mut files = entries
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                Path::new(filename),
                Path::new("a"),
                Path::new("b"),
                Path::new("c"),
                Path::new("d")
            ]
        );

        let entries = fs::read_dir("./d").unwrap();
        let mut files = entries
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()
            .unwrap();
        files.sort();
        assert_eq!(files, vec![Path::new("d/e")]);
    }

    fn reset(path: &Path) {
        fs::create_dir("d").unwrap();
        let files = vec!["a", "b", "c", "d/e"];

        let tar_gz = File::create(path).unwrap();
        let mut tar = Builder::new(tar_gz);

        for file in files {
            let mut f = File::create(file).unwrap();
            f.write_all(b"*").unwrap();

            tar.append_path(file).unwrap();

            fs::remove_file(file).unwrap();
        }

        let tar_gz = File::open(path).unwrap();
        let mut tar = Archive::new(tar_gz);
        tar.unpack(".").unwrap();
    }

    #[rstest]
    #[case("tar xvf foo.tar", "mkdir -p foo && tar xvf foo.tar -C foo", true)]
    #[case("tar -xvf bar.tar", "mkdir -p bar && tar -xvf bar.tar -C bar", true)]
    #[case(
        "tar --extract -f baz.tar",
        "mkdir -p baz && tar --extract -f baz.tar -C baz",
        true
    )]
    fn test_match(#[case] command: &str, #[case] fixed: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some("".to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("tar xvf foo.tar", "mkdir -p foo && tar xvf foo.tar -C foo")]
    #[case("tar -xvf bar.tar", "mkdir -p bar && tar -xvf bar.tar -C bar")]
    #[case(
        "tar --extract -f baz.tar",
        "mkdir -p baz && tar --extract -f baz.tar -C baz"
    )]
    fn test_side_effect(#[case] command: &str, #[case] fixed: &str) {
        let mut command = CrabCommand::new(command.to_owned(), Some("".to_owned()), None);
        side_effect(command, &"".to_owned());
        assert_eq!(
            fs::read_dir(Path::new(&command.script_parts[2]))
                .unwrap()
                .count(),
            1
        );
    }

    #[rstest]
    #[case("tar xvf foo.tar", "mkdir -p foo && tar xvf foo.tar -C foo", vec!["mkdir -p foo", "tar xvf foo.tar -C foo"])]
    #[case("tar -xvf bar.tar", "mkdir -p bar && tar -xvf bar.tar -C bar", vec!["mkdir -p bar", "tar -xvf bar.tar -C bar"])]
    #[case("tar --extract -f baz.tar", "mkdir -p baz && tar --extract -f baz.tar -C baz", vec!["mkdir -p baz", "tar --extract -f baz.tar -C baz"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] fixed: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some("".to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
