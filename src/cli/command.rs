use shellwords;
use std::process::{Command, Stdio};
use std::str;

pub struct CorrectedCommand {
    pub script: String,
    pub side_effect: Option<fn(CrabCommand, &String)>,
    pub priority: u16,
}

impl CorrectedCommand {
    pub fn new(
        script: String,
        side_effect: Option<fn(CrabCommand, &String)>,
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
            (side_effect)(old_command, &self.script);
        }
        println!("{}", self.get_script());
    }
}

pub struct CrabCommand {
    pub script: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub script_parts: Vec<String>,
}

impl CrabCommand {
    pub fn new(script: String, stdout: Option<String>, stderr: Option<String>) -> Self {
        let split_command = CrabCommand::split_command(&script);

        CrabCommand {
            script,
            stdout,
            stderr,
            script_parts: split_command,
        }
    }

    fn split_command(command: &str) -> Vec<String> {
        // Split the command using shell-like syntax.
        shellwords::split(command).expect("")
    }

    // pub fn get_corrected_commands(self) -> Vec<Self> {
    //     let corrected_commands: Vec<Self> = Vec::new();
    //     for rule in rules::get_rules() {}
    //     return corrected_commands;
    // }

    // pub fn script_parts(self) -
}

pub fn run_command(raw_command: Vec<String>) -> CrabCommand {
    let command = prepare_command(raw_command);

    let mut output = shell_command()
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
    return CrabCommand::new(command, stdout, stderr);
}

fn prepare_command(raw_command: Vec<String>) -> String {
    // TODO: Expand aliases (`shell.from_shell()`)
    raw_command.join(" ").trim().to_owned()
}

pub fn shell_command() -> Command {
    // TODO: Retrieve the shell type from the config
    // let words_str = CONFIG.shell();
    let words_str = "bash".to_string();
    let mut words_vec = shellwords::split(&words_str).expect("empty shell command");
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
