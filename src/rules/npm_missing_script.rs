use super::{
    utils::{match_rule_with_is_app, npm::is_npm_available},
    Rule,
};
use crate::{
    cli::command::CrabCommand,
    rules::utils::npm::{mockable_get_scripts, run_npm_command},
    shell::Shell,
    utils::replace_command,
};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command
            .script_parts
            .iter()
            .any(|part| part.starts_with("ru"))
            && output.contains("npm ERR! missing script: ")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["npm"], None)
}

pub fn mockable_get_new_command<F>(command: &mut CrabCommand, fn_get_scripts: F) -> Vec<String>
where
    F: Fn() -> Vec<u8>,
{
    if let Some(output) = &command.output {
        let re = Regex::new(r".*missing script: (.*)\n").unwrap();
        if let Some(caps) = re.captures(output) {
            println!("{caps:?}");
            replace_command(
                command,
                &caps[1],
                mockable_get_scripts(fn_get_scripts)
                    .iter()
                    .map(|s| s.as_str())
                    .collect(),
            )
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    mockable_get_new_command(command, run_npm_command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "npm_missing_script".to_owned(),
        Some(is_npm_available()),
        None,
        None,
        match_rule,
        get_new_command,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::{match_rule, mockable_get_new_command};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;
    use rstest::rstest;

    fn output(script: &str) -> String {
        format!(
            r#"
npm ERR! Linux 4.4.0-31-generic
npm ERR! argv "/opt/node/bin/node" "/opt/node/bin/npm" "run" "dvelop"
npm ERR! node v4.4.7
npm ERR! npm  v2.15.8

npm ERR! missing script: {script}
npm ERR!
npm ERR! If you need help, you may report this error at:
npm ERR!     <https://github.com/npm/npm/issues>

npm ERR! Please include the following file with any support request:
npm ERR!     /home/nvbn/exp/code_view/client_web/npm-debug.log
"#
        )
    }
    #[rstest]
    #[case("npm ru wach", &output("wach"), true)]
    #[case("npm run live-tes", &output("live-tes"), true)]
    #[case("npm run-script sahare", &output("sahare"), true)]
    #[case("npm wach", &output("wach"), false)]
    #[case("vim live-tes", &output("live-tes"), false)]
    #[case("npm run-script sahare", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("npm ru wach-tests", &output("wach-tests"), vec!["npm ru watch-test"])]
    #[case("npm -i run-script dvelop", &output("dvelop"), vec!["npm -i run-script develop", "npm -i run-script build"])]
    #[case("npm -i run-script buld -X POST", &output("buld"), vec!["npm -i run-script build -X POST", "npm -i run-script develop -X POST"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
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
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(
            mockable_get_new_command(&mut command, || run_script_stdout.to_vec()),
            expected
        );
    }
}
