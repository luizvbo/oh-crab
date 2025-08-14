use shlex::{split, Shlex};
use std::process::{Command, Stdio};
use std::{fmt, str};

use crate::shell::Shell;

#[derive(Debug)]
pub struct CorrectedCommand {
    pub script: String,
    pub side_effect: Option<fn(CrabCommand, Option<&str>)>,
    pub priority: u16,
}

impl CorrectedCommand {
    pub fn new(
        script: String,
        side_effect: Option<fn(CrabCommand, Option<&str>)>,
        priority: u16,
    ) -> Self {
        Self {
            script,
            side_effect,
            priority,
        }
    }
    pub fn get_script(&self) -> &String {
        &self.script
    }
    pub fn run(&self, old_command: CrabCommand) {
        if let Some(side_effect) = self.side_effect {
            (side_effect)(old_command, Some(&self.script));
        }
        println!("{}", self.get_script());
    }
}

#[derive(Debug)]
pub struct CrabCommand {
    pub script: String,
    pub output: Option<String>,
    pub script_parts: Vec<String>,
}

impl fmt::Display for CrabCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "script: {}, output: {}",
            self.script,
            self.output.as_ref().unwrap_or(&"".to_owned()),
        )
    }
}

fn concat_stdout_stderrr(stdout: Option<String>, stderr: Option<String>) -> Option<String> {
    match (stdout, stderr) {
        (Some(stdout), Some(stderr)) => Some({
            if !stderr.is_empty() && !stdout.is_empty() {
                format!("{}\n{}", stdout, &stderr)
            } else if !stderr.is_empty() {
                stderr
            } else {
                stdout
            }
        }),
        (Some(stdout), None) => Some(stdout),
        (None, Some(stderr)) => Some(stderr),
        (None, None) => None,
    }
}

impl CrabCommand {
    pub fn new(script: String, stdout: Option<String>, stderr: Option<String>) -> Self {
        let split_parts = CrabCommand::split_command(&script);
        let output = concat_stdout_stderrr(stdout, stderr);

        CrabCommand {
            script,
            output,
            script_parts: split_parts,
        }
    }

    pub fn update(
        &self,
        script: Option<String>,
        stdout: Option<String>,
        stderr: Option<String>,
    ) -> CrabCommand {
        let output = concat_stdout_stderrr(stdout, stderr);

        CrabCommand::new(
            script.unwrap_or(self.script.to_owned()),
            output.map_or(self.output.to_owned(), Some),
            None,
        )
    }

    fn split_command(command: &str) -> Vec<String> {
        // Split the command using shell-like syntax.
        shlex_split(command)
    }
}

/// Differently from shlex original split function, this function always returns the
/// resulting vector, even if there's an error.
pub fn shlex_split(in_str: &str) -> Vec<String> {
    let mut shl = Shlex::new(in_str);
    shl.by_ref().collect()
}

pub fn run_command(raw_command: Vec<String>, system_shell: &dyn Shell) -> CrabCommand {
    let command = prepare_command(raw_command);
    let mut output = shell_command(&system_shell.get_shell())
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Command failed to start")
        .wait_with_output()
        .expect("failed to wait on child");

    let exit_status = output.status;
    // output.status.success();
    let stdout = str::from_utf8(&output.stdout).map(|s| s.to_owned()).ok();
    let stderr = str::from_utf8(&output.stderr).map(|s| s.to_owned()).ok();
    CrabCommand::new(command, stdout, stderr)
}

fn prepare_command(raw_command: Vec<String>) -> String {
    // TODO: Expand aliases (`shell.from_shell()`)
    raw_command.join(" ").trim().to_owned()
}

pub fn shell_command(words_str: &str) -> Command {
    let mut words_vec = split(words_str).expect("empty shell command");
    let mut words = words_vec.iter_mut();
    let first_cmd = words.next().expect("absent shell binary");
    let dash_c = if words_str.contains("cmd.exe") {
        "/c"
    } else {
        "-c"
    };
    let mut cmd = Command::new(first_cmd);
    cmd.args(words);
    cmd.arg(dash_c);
    cmd
}

#[cfg(test)]
mod tests {
    use crate::{
        cli::command::shell_command,
        shell::{Bash, Shell},
    };

    use super::run_command;

    #[test]
    fn test_shell_command() {
        let shell_name = "bash".to_owned();
        let cmd = shell_command(&shell_name);
        assert_eq!(cmd.get_args().len(), 1);
        assert_eq!(cmd.get_program().to_str().unwrap(), shell_name);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_run_command() {
        let terminal_command = {
            if cfg!(target_family = "unix") {
                "echo"
            } else {
                "Write-Output"
            }
        };
        let command_vec = vec![terminal_command.to_owned(), "Hello!".to_owned()];
        let command = command_vec.join(" ").trim().to_owned();
        let system_shell: Box<dyn Shell> = Box::new(Bash {});
        let crab_command = run_command(command_vec, &*system_shell);
        assert_eq!(crab_command.script, command);
        assert_eq!(crab_command.output.unwrap(), "Hello!\n");
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_run_command_with_error() {
        let command_vec = vec!["ls".to_owned(), "non_existent_directory".to_owned()];
        let command = command_vec.join(" ").trim().to_owned();
        let system_shell: Box<dyn Shell> = Box::new(Bash {});
        let crab_command = run_command(command_vec, &*system_shell);
        assert_eq!(crab_command.script, command);
        assert!(crab_command
            .output
            .unwrap()
            .contains("No such file or directory"));
    }
}
