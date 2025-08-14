// FILE: ./src/cli/parser.rs
use clap::{command, Arg, ArgAction};
use std::env;

use crate::{ARGUMENT_PLACEHOLDER, ENV_VAR_NAME_ALIAS, ENV_VAR_NAME_HISTORY, ENV_VAR_NAME_SHELL};

/// Prepares arguments by:
/// - Removing placeholder and moving arguments after it to beginning, we need this
///   to distinguish arguments from `command` with ours;
/// - Adding `--` before `command`, so that our parser ignores arguments of `command`.
///
/// * `argv`:
pub fn prepare_arguments(mut argv: Vec<String>) -> Vec<String> {
    if let Some(index) = argv.iter().position(|x| *x == ARGUMENT_PLACEHOLDER) {
        let mut command_part = argv.split_off(index);
        // Remove the placeholder itself
        command_part.remove(0);

        let mut processed_args = command_part;
        processed_args.push("--".to_owned());
        processed_args.append(&mut argv);
        processed_args
    } else {
        if !argv.is_empty() && !argv[0].starts_with('-') && argv[0] != "--" {
            argv.insert(0, "--".to_owned());
        }
        argv
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
                .help("Prints the shell function using the given alias")
                .required(false)
                .env(ENV_VAR_NAME_ALIAS)
                .default_value("crab"),
        )
        .arg(
            Arg::new("shell")
                .long("shell")
                .short('s')
                .help("Shell used to call ohcrab")
                .env(ENV_VAR_NAME_SHELL)
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
            Arg::new("select-first")
                .long("select-first")
                .short('y')
                .help("Automatically select the first corrected command")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("command")
                .help("Command that should be fixed")
                .action(ArgAction::Append)
                .required(false)
                .env(ENV_VAR_NAME_HISTORY)
                .last(true),
        )
}

#[cfg(test)]
mod tests {
    use crate::{
        cli::parser::{get_parser, prepare_arguments},
        ARGUMENT_PLACEHOLDER, ENV_VAR_NAME_ALIAS, ENV_VAR_NAME_HISTORY, ENV_VAR_NAME_SHELL,
    };
    use clap::parser::ValueSource;
    use std::env;

    // This single test now covers all logic related to the 'alias' argument,
    // preventing race conditions.
    #[test]
    fn test_parser_alias_logic() {
        // Test default value
        env::remove_var(ENV_VAR_NAME_ALIAS);
        let matches = get_parser().get_matches_from(Vec::<String>::new());
        assert_eq!(
            matches.value_source("alias"),
            Some(ValueSource::DefaultValue)
        );
        assert_eq!(
            matches.get_one::<String>("alias"),
            Some(&"crab".to_string())
        );

        // Test value from environment variable
        env::set_var(ENV_VAR_NAME_ALIAS, "env_alias");
        let matches = get_parser().get_matches_from(Vec::<String>::new());
        assert_eq!(
            matches.value_source("alias"),
            Some(ValueSource::EnvVariable)
        );
        assert_eq!(
            matches.get_one::<String>("alias"),
            Some(&"env_alias".to_string())
        );

        // Test value from command line (overrides env var)
        let matches = get_parser().get_matches_from(vec!["--alias", "new_alias"]);
        assert_eq!(
            matches.value_source("alias"),
            Some(ValueSource::CommandLine)
        );
        assert_eq!(
            matches.get_one::<String>("alias"),
            Some(&"new_alias".to_string())
        );

        // Cleanup
        env::remove_var(ENV_VAR_NAME_ALIAS);
    }

    // This test is now independent and doesn't interfere.
    #[test]
    fn test_parser_other_arguments() {
        env::remove_var(ENV_VAR_NAME_SHELL);
        env::remove_var(ENV_VAR_NAME_HISTORY);

        // Test default shell
        assert_eq!(
            get_parser()
                .get_matches_from(Vec::<String>::new())
                .get_one::<String>("shell"),
            Some(&"bash".to_string())
        );

        // Test shell from command line
        assert_eq!(
            get_parser()
                .get_matches_from(vec!["-s", "zsh"])
                .get_one::<String>("shell"),
            Some(&"zsh".to_string())
        );

        // Test shell from environment variable
        env::set_var(ENV_VAR_NAME_SHELL, "pws");
        assert_eq!(
            get_parser()
                .get_matches_from(Vec::<String>::new())
                .get_one::<String>("shell"),
            Some(&"pws".to_string())
        );

        // Test debug flag
        assert!(get_parser()
            .get_matches_from(vec!["-d", "--", "anything"])
            .get_flag("debug"));

        // Test command from command line
        assert_eq!(
            get_parser()
                .get_matches_from(vec!["--", "ls", "-a"])
                .get_many::<String>("command")
                .unwrap()
                .collect::<Vec<_>>(),
            ["ls", "-a"]
        );

        // Test command from environment variable
        env::set_var(ENV_VAR_NAME_HISTORY, "ls -a\nls -lah");
        assert_eq!(
            get_parser()
                .get_matches_from(Vec::<String>::new())
                .get_many::<String>("command")
                .unwrap()
                .collect::<Vec<_>>(),
            ["ls -a\nls -lah"]
        );

        // Cleanup
        env::remove_var(ENV_VAR_NAME_SHELL);
        env::remove_var(ENV_VAR_NAME_HISTORY);
    }

    #[test]
    fn test_prepare_arguments() {
        for (input, exp_output) in [
            (
                vec![
                    "arg1".to_owned(),
                    "arg2".to_owned(),
                    ARGUMENT_PLACEHOLDER.to_owned(),
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
    /// - ARGUMENT_PLACEHOLDER
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
            ARGUMENT_PLACEHOLDER.to_owned(),
            "--shell".to_owned(),
            "custom_bash".to_owned(),
        ]);
        let mut vec_matches = get_parser().get_matches_from(prepared_args);
        let command = vec_matches
            .remove_many::<String>("command")
            .expect("Command not found")
            .collect::<Vec<_>>();
        let shell_type = vec_matches.remove_one::<String>("shell");
        assert_eq!(command, ["arg1", "arg2"]);
        assert_eq!(shell_type, Some("custom_bash".to_owned()));
    }
}
