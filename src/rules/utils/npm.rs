use regex::Regex;
use std::process::Command;
use which::which;

pub fn is_npm_available() -> bool {
    which("npm").is_ok()
}

pub fn run_npm_command() -> Vec<u8> {
    let output = Command::new("npm")
        .arg("run-script")
        .output()
        .expect("Failed to execute command");
    output.stdout
}

pub fn mockable_get_scripts<'a, F>(fn_run_npm_command: F) -> Vec<String>
where
    F: Fn() -> Vec<u8>,
{

    let npm_output = fn_run_npm_command();
    let stdout = String::from_utf8_lossy(&npm_output);
    let mut should_yield = false;
    let mut scripts = Vec::new();
    let re = Regex::new(r"^  \S+").unwrap();

    for line in stdout.lines() {
        if line.contains("available via `npm run-script`:") {
            should_yield = true;
            continue;
        }

        if should_yield && re.is_match(line) {
            let script = line.trim().split_whitespace().next().unwrap().to_string();
            scripts.push(script);
        }
    }

    scripts
}
pub fn get_scripts() -> Vec<String> {
    mockable_get_scripts(run_npm_command)
}

#[cfg(test)]
mod tests {
    use crate::rules::utils::npm::mockable_get_scripts;

    #[test]
    fn test_get_scripts() {
        let run_script_stdout = b"
Lifecycle scripts included in code-view-web:
  test
    jest

available via `npm run-script`:
  build
    cp node_modules/ace-builds/src-min/ -a resources/ace/ && webpack --progress --colors -p --config ./webpack.production.config.js
  develop
    cp node_modules/ace-builds/src/ -a resources/ace/ && webpack-dev-server --progress --colors
  watch-test
    jest --verbose --watch

";
        let scripts = mockable_get_scripts(|| run_script_stdout.to_vec());
        assert_eq!(scripts, vec!["build", "develop", "watch-test"]);
    }
}
