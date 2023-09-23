use clap::{command, parser::ValueSource, Arg, ArgAction};

use std::env;

use crate::{
    cli::command::{run_command, CrabCommand},
    rules::{get_corrected_commands, get_rules, selected_command},
    shell::{get_bash_type, Shell},
    shell::{Bash, Zsh},
};

const ARGUMENT_PLACEHOLDER: &str = "OHCRAB_ARGUMENT_PLACEHOLDER";


/// Prepares arguments by:
/// - Removing placeholder and moving arguments after it to beginning, we need this
///     to distinguish arguments from `command` with ours;
/// - Adding `--` before `command`, so that our parser ignores arguments of `command`.
///
/// * `argv`:
pub fn prepare_arguments(mut argv: Vec<String>) -> Vec<String> {
    match argv.iter().position(|x| x == &ARGUMENT_PLACEHOLDER) {
        Some(index) => {
            let mut argv_processed = Vec::<String>::with_capacity(argv.len() + 1);
            argv_processed.extend_from_slice(&argv[index + 1..]);
            argv_processed.push("--".to_owned());
            argv_processed.extend_from_slice(&argv[..index]);
            argv_processed
        }
        None => {
            if argv.len() > 0 && !argv[0].starts_with('-') && argv[0] != "--" {
                argv.insert(0, "--".to_owned());
            }
            argv
        }
    }
}

/// Generate an argument parser using clap
pub fn get_parser() -> clap::Command {
    command!()
        .no_binary_name(true)
        .arg(
            Arg::new("alias")
                .long("alias")
                .short('a')
                .help("[custom-alias-name] Prints alias for current shell")
                .required(false)
                .env("OC_ALIAS")
                .default_value("crab"),
        )
        .arg(
            Arg::new("shell")
                .long("shell")
                .short('s')
                .help("Shell used to call ohcrab")
                .required(false)
                .default_value("bash"),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .help("Enable debug output")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("command")
                .help("Command that should be fixed")
                .action(ArgAction::Append)
                .required(false)
                .last(true),
        )
}

#[cfg(test)]
mod tests {
    use clap::parser::ValueSource;

    use crate::cli::parser::{get_parser, prepare_arguments};
    use std::env;

    #[test]
    fn test_get_parser_alias_source() {
        assert_eq!(
            get_parser()
                .get_matches_from(Vec::<String>::new())
                .value_source("alias"),
            Some(ValueSource::DefaultValue)
        );
        env::set_var("OC_ALIAS", "env_alias");
        assert_eq!(
            get_parser()
                .get_matches_from(Vec::<String>::new())
                .value_source("alias"),
            Some(ValueSource::EnvVariable)
        );
        assert_eq!(
            get_parser()
                .get_matches_from(vec!["--alias=new_alias"])
                .value_source("alias"),
            Some(ValueSource::CommandLine)
        );
    }
    #[test]
    fn test_get_parser_matches() {
        // In case no alias is provided or the environment variable `OC_ALIAS`
        // is not set we should get the default value "crab"
        assert_eq!(
            get_parser()
                .get_matches_from(Vec::<String>::new())
                .get_one::<String>("alias"),
            Some(&"crab".to_string())
        );
        // Test alias defined from environment variable
        env::set_var("OC_ALIAS", "env_alias");
        assert_eq!(
            get_parser()
                .get_matches_from(Vec::<String>::new())
                .get_one::<String>("alias"),
            Some(&"env_alias".to_string())
        );
        assert_eq!(
            get_parser()
                .get_matches_from(vec!["--alias", "new_alias"])
                .get_one::<String>("alias"),
            Some(&"new_alias".to_string())
        );
        assert_eq!(
            get_parser()
                .get_matches_from(Vec::<String>::new())
                .get_one::<String>("shell"),
            Some(&"bash".to_string())
        );
        assert_eq!(
            get_parser()
                .get_matches_from(vec!["-s", "bash"])
                .get_one::<String>("shell"),
            Some(&"bash".to_string())
        );
        assert_eq!(
            get_parser()
                .get_matches_from(vec!["--shell", "bash"])
                .get_one::<String>("shell"),
            Some(&"bash".to_string())
        );
        assert_eq!(
            get_parser()
                .get_matches_from(vec!["-d", "--", "anything"])
                .get_flag("debug"),
            true
        );
        assert_eq!(
            get_parser()
                .get_matches_from(vec!["--", "my", "command"])
                .get_many::<String>("command")
                .expect("Command not found")
                .collect::<Vec<_>>(),
            ["my", "command"]
        );
    }

    #[test]
    fn test_prepare_arguments() {
        for (input, exp_output) in [
            (
                vec![
                    "arg1".to_owned(),
                    "arg2".to_owned(),
                    "OHCRAB_ARGUMENT_PLACEHOLDER".to_owned(),
                    "arg3".to_owned(),
                ],
                vec!["arg3", "--", "arg1", "arg2"],
            ),
            (
                vec!["arg1".to_owned(), "arg2".to_owned(), "arg3".to_owned()],
                vec!["--", "arg1", "arg2", "arg3"],
            ),
            (
                vec!["-param".to_owned(), "arg2".to_owned(), "arg3".to_owned()],
                vec!["-param", "arg2", "arg3"],
            ),
        ] {
            assert_eq!(prepare_arguments(input), exp_output);
        }
    }


    /// Tests the argument processing logic.
    ///
    /// This test checks if the argument processing functions work as expected. It prepares some
    /// arguments, processes them, and verifies that the resulting command and shell type are as
    /// expected.
    ///
    /// The arguments used in this test are:
    /// - "arg1"
    /// - "arg2"
    /// - "OHCRAB_ARGUMENT_PLACEHOLDER"
    /// - "--shell"
    /// - "custom_bash"
    ///
    /// The expected behavior is:
    /// - The resulting command should be ["arg1", "arg2"].
    /// - The shell type should be "custom_bash".
    #[test]
    fn test_process_arguments() {
        let prepared_args = prepare_arguments(vec![
            "arg1".to_owned(),
            "arg2".to_owned(),
            "OHCRAB_ARGUMENT_PLACEHOLDER".to_owned(),
            "--shell".to_owned(),
            "custom_bash".to_owned(),
        ]);
        let mut vec_matches = get_parser()
            .get_matches_from(&prepared_args);
        let command = vec_matches
            .remove_many::<String>("command")
            .expect("Command not found")
            .collect::<Vec<_>>();
        let shell_type = vec_matches.remove_one::<String>("shell");
        println!("{:?}", command);
        assert_eq!(command, ["arg1", "arg2"]);
        assert_eq!(shell_type, Some("custom_bash".to_owned()));
    }
}
