use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{ARGUMENT_PLACEHOLDER, ENV_VAR_NAME_ALIAS, ENV_VAR_NAME_HISTORY, ENV_VAR_NAME_SHELL};

pub trait Shell {
    fn app_alias(&self, alias_name: &str) -> String;
    fn get_shell(&self) -> String;
    fn get_history_file_name(&self) -> String;
    fn script_from_history(&self, command_script: &str) -> String {
        command_script.to_owned()
    }

    fn get_history(&self, file_path: Option<&str>) -> Vec<String> {
        let history_file_name = {
            if let Some(path) = file_path {
                path.to_owned()
            } else {
                self.get_history_file_name()
            }
        };
        let mut history: Vec<String> = Vec::new();
        if Path::new(&history_file_name).exists() {
            if let Ok(file) = File::open(&history_file_name) {
                let reader = io::BufReader::new(file);

                // TODO: Limit history length based on settings
                for line in reader.lines() {
                    let prepared = self.script_from_history(&line.unwrap_or("".to_owned()));
                    let prepared = prepared.trim();
                    if !prepared.is_empty() {
                        history.push(prepared.to_owned());
                    }
                }
            }
        }
        history
    }

    fn get_builtin_commands(&self) -> Vec<String> {
        vec![
            "alias".to_owned(),
            "bg".to_owned(),
            "bind".to_owned(),
            "break".to_owned(),
            "builtin".to_owned(),
            "case".to_owned(),
            "cd".to_owned(),
            "command".to_owned(),
            "compgen".to_owned(),
            "complete".to_owned(),
            "continue".to_owned(),
            "declare".to_owned(),
            "dirs".to_owned(),
            "disown".to_owned(),
            "echo".to_owned(),
            "enable".to_owned(),
            "eval".to_owned(),
            "exec".to_owned(),
            "exit".to_owned(),
            "export".to_owned(),
            "fc".to_owned(),
            "fg".to_owned(),
            "getopts".to_owned(),
            "hash".to_owned(),
            "help".to_owned(),
            "history".to_owned(),
            "if".to_owned(),
            "jobs".to_owned(),
            "kill".to_owned(),
            "let".to_owned(),
            "local".to_owned(),
            "logout".to_owned(),
            "popd".to_owned(),
            "printf".to_owned(),
            "pushd".to_owned(),
            "pwd".to_owned(),
            "read".to_owned(),
            "readonly".to_owned(),
            "return".to_owned(),
            "set".to_owned(),
            "shift".to_owned(),
            "shopt".to_owned(),
            "source".to_owned(),
            "suspend".to_owned(),
            "test".to_owned(),
            "times".to_owned(),
            "trap".to_owned(),
            "type".to_owned(),
            "typeset".to_owned(),
            "ulimit".to_owned(),
            "umask".to_owned(),
            "unalias".to_owned(),
            "unset".to_owned(),
            "until".to_owned(),
            "wait".to_owned(),
            "while".to_owned(),
        ]
    }
}

pub fn get_bash_type(shell_type: &str) -> Box<dyn Shell> {
    let shell_candidate = shell_type.to_lowercase();
    match shell_candidate.as_str() {
        "zsh" => Box::new(Zsh),
        "bash" => Box::new(Bash),
        _ => panic!("The shell '{}' is not supported yet", shell_type),
    }
}

pub struct Zsh;
pub struct Bash;

impl Shell for Zsh {
    fn get_shell(&self) -> String {
        "zsh".to_owned()
    }

    fn app_alias(&self, alias_name: &str) -> String {
        format!(
            r#"
            {alias_name} () {{
                export {var_name_shell}="zsh";
                export {var_name_alias}="{alias_name}";
                export {var_name_history}="$(fc -ln -1)";
                OC_CMD=$(
                    ohcrab {argument_placeholder} $@
                ) && eval $OC_CMD;
                unset {var_name_history};
            }}
            "#,
            alias_name = alias_name,
            var_name_shell = ENV_VAR_NAME_SHELL,
            var_name_alias = ENV_VAR_NAME_ALIAS,
            var_name_history = ENV_VAR_NAME_HISTORY,
            argument_placeholder = ARGUMENT_PLACEHOLDER,
        )
    }

    fn script_from_history(&self, command_script: &str) -> String {
        if command_script.contains(';') {
            command_script.split_once(';').unwrap().1.to_owned()
        } else {
            "".to_owned()
        }
    }

    fn get_history_file_name(&self) -> String {
        match env::var("HISTFILE") {
            Ok(val) => val,
            Err(_) => dirs::home_dir()
                .unwrap()
                .join(".zsh_history")
                .to_str()
                .unwrap()
                .to_string(),
        }
    }
}

impl Shell for Bash {
    fn get_shell(&self) -> String {
        "bash".to_owned()
    }

    fn app_alias(&self, alias_name: &str) -> String {
        format!(
            r#"
            function {alias_name} () {{
                export {var_name_shell}="bash";
                export {var_name_alias}="{alias_name}";
                export {var_name_history}="$(fc -ln -1)";
                OC_CMD=$(
                    ohcrab {argument_placeholder} "$@"
                ) && eval "$OC_CMD";
                unset {var_name_history};
            }}
            "#,
            alias_name = alias_name,
            var_name_history = ENV_VAR_NAME_HISTORY,
            var_name_shell = ENV_VAR_NAME_SHELL,
            var_name_alias = ENV_VAR_NAME_ALIAS,
            argument_placeholder = ARGUMENT_PLACEHOLDER,
        )
    }

    fn get_history_file_name(&self) -> String {
        match env::var("HISTFILE") {
            Ok(val) => val,
            Err(_) => dirs::home_dir()
                .unwrap()
                .join(".bash_history")
                .to_str()
                .unwrap()
                .to_string(),
        }
    }
}

#[cfg(test)]
mod test_zsh {
    use crate::shell::Shell;
    use std::io::{self, Write};
    use tempfile::NamedTempFile;

    use super::Zsh;

    #[test]
    fn test_get_history() {
        // Create a file inside of `std::env::temp_dir()`.
        let mut file = NamedTempFile::new().unwrap();

        writeln!(
            file,
            ": 1702325001:0;ls -lah\n: 1702325001:0;cd /tmp\n: 1702325001:0;nvim"
        )
        .unwrap();
        let path = file.path().to_str().unwrap();

        let system_shell = Zsh {};
        assert_eq!(
            system_shell.get_history(Some(path)),
            vec!["ls -lah", "cd /tmp", "nvim"]
        );
    }
}
