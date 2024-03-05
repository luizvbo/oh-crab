#[cfg(test)]
mod tests {
    use crate::ENV_VAR_NAME_SHELL;
    use regex::Regex;
    use rstest::rstest;
    use core::panic;
    use std::env;
    use std::process::{Command, Stdio};
    use std::str;
    use std::thread;
    use std::time::Duration;

    #[rstest]
    #[case("cd abcdef", "mkdir -p abcdef && cd abcdef")]
    fn test_match(
        #[case] command: &str,
        #[case] expected_output: &str,
    ) {
        env::set_var(ENV_VAR_NAME_SHELL, "bash");
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg(command)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Command failed to start");

        thread::sleep(Duration::from_secs(1));
        child
            .kill()
            .expect("Failed to send SIGINT to child process");

        let output = child.wait_with_output().expect("Failed to wait on child");
        let stdout = str::from_utf8(&output.stdout).unwrap();

        let re = Regex::new(r#"\nCandidate command\(s\): \["(.*?)"\]"#).unwrap();
        if let Some(caps) = re.captures(stdout) {
            assert_eq!(caps.get(1).unwrap().as_str(), expected_output);
        }
        else {
            panic!("It was not possible to get the command generated from: \"{}\"", stdout)
        }
    }
}
