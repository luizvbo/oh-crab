use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
use shlex::Quoter;
use std::fs;
use std::fs::File;
use std::io::Result;
use zip::read::ZipArchive;

fn is_bad_zip(file: &str) -> Result<bool> {
    let reader = File::open(file)?;
    let mut archive = ZipArchive::new(reader)?;

    Ok(archive.len() > 1)
}

fn get_zipped_file(command: &CrabCommand) -> Option<String> {
    // unzip works this way:
    // unzip [-flags] file[.zip] [file(s) ...] [-x file(s) ...]
    //                ^          ^ files to unzip from the archive
    //                archive to unzip
    for c in &command.script_parts[1..] {
        if !c.starts_with('-') {
            if c.ends_with(".zip") {
                return Some(c.to_string());
            } else {
                return Some(format!("{}.zip", c));
            }
        }
    }
    None
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if command.script.contains("-d") {
        return false;
    }
    let zipped_files = get_zipped_file(command);
    return match zipped_files {
        Some(zip_file) => is_bad_zip(&zip_file).is_ok(),
        None => false,
    };
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["unzip"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let shlex_quoter = Quoter::new();
    let zipped_file = get_zipped_file(command);
    return match zipped_file {
        Some(zipped_file) => vec![format!(
            "{} -d {}",
            command.script,
            shlex_quoter.quote(&zipped_file).unwrap()
        )],
        None => vec![],
    };
}

pub fn side_effect(old_cmd: CrabCommand, command: Option<&str>) {
    let zipped_file = get_zipped_file(&old_cmd);
    if let Some(zipped_file) = zipped_file {
        match fs::File::open(&zipped_file) {
            Ok(reader) => match ZipArchive::new(reader) {
                Ok(mut archive) => {
                    for i in 0..archive.len() {
                        match archive.by_index(i) {
                            Ok(mut file) => {
                                let outpath = file.mangled_name();
                                match outpath.canonicalize() {
                                    Ok(outpath) => {
                                        if !outpath.starts_with(std::env::current_dir().unwrap()) {
                                            // it's unsafe to overwrite files outside of the current directory
                                            continue;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to canonicalize path: {}", e);
                                        continue;
                                    }
                                }

                                if outpath.is_file() {
                                    if let Err(e) = fs::remove_file(&outpath) {
                                        eprintln!(
                                            "Failed to remove file {}: {}",
                                            outpath.display(),
                                            e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to get file from archive: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open zip archive: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Failed to open file {}: {}", zipped_file, e);
            }
        }
    }
}
pub fn get_rule() -> Rule {
    Rule::new(
        "dirty_unzip".to_owned(),
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
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;
    use zip::{write::SimpleFileOptions, ZipWriter};

    pub fn zip_error(filename: &str, tmp_dir: &TempDir) {
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
        let file = File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);

        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file("a", options).unwrap();
        zip.write_all(b"1").unwrap();

        zip.start_file("b", options).unwrap();
        zip.write_all(b"2").unwrap();

        zip.start_file("c", options).unwrap();
        zip.write_all(b"3").unwrap();

        zip.start_file("d/e", options).unwrap();
        zip.write_all(b"4").unwrap();

        zip.finish().unwrap();
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
                    zip_error(&unquoted(ext), &tmp_dir);
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
