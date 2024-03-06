#[cfg(test)]
mod tests {
    use crate::ENV_VAR_NAME_SHELL;
    use regex::Regex;
    use rstest::rstest;
    use std::env;
    use std::process::Stdio;
    use std::str;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command as TokioCommand;

    #[rstest]
    #[case("cd abcdef", "mkdir -p abcdef && cd abcdef")]
    #[tokio::test]
    async fn test_match(
        #[case] command: &str,
        #[case] expected_output: &str,
    ) {
        env::set_var(ENV_VAR_NAME_SHELL, "bash");
        let mut child = TokioCommand::new("cargo")
            .arg("run")
            .arg("--")
            .arg(command)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Command failed to start");

        let reader = BufReader::new(child.stdout.take().expect("child did not have a handle to stdout"));
        let mut lines = reader.lines();
        let mut command_line = "".to_owned();

        while let Some(line) = lines.next_line().await.expect("Failed to read line from child stdout") {
            if line.starts_with("Candidate command(s): [") {
                command_line = line.clone();
                child.kill().await.expect("Failed to send SIGINT to child process");
                break;
            }
        }

        let re = Regex::new(r#"Candidate command\(s\): \["(.*)"\]"#).unwrap();
        println!("{:?}", re.captures(&command_line));
        if let Some(caps) = re.captures(&command_line) {
            assert_eq!(caps.get(1).unwrap().as_str(), expected_output);
        }
        else {
            panic!("It was not possible to get the command from the line: \"{}\"", command_line);
        }
    }
}
