pub mod command;
use clap::{command, Arg, ArgAction};

use std::env;

use crate::command::run_command;

const ARGUMENT_PLACEHOLDER: &str = "OHCRAB_ARGUMENT_PLACEHOLDER";

pub fn handler() {
    // Skip the first element of `env::args()` (the name of program)
    let args = env::args().skip(1).collect();
    let args = prepare_arguments(args);
    let mut arg_matches = get_parser().get_matches_from(&args);

    if let Some(command) = arg_matches.remove_many::<String>("command") {
        let crab_command = run_command(command.collect());
        let corrected_commands = crab_command.get_corrected_commands();
    } else {
        if let Some(alias) = arg_matches.get_one::<String>("alias") {
            panic!("Alias support not implemented yet");
        } else {
            return;
        }
    }
}

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

#[test]
fn test_get_parser() {
    env::set_var("OC_ALIAS", "env_alias");
    // Test alias defined from environment variable
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
            .get_matches_from(vec!["-a", "new_alias"])
            .get_one::<String>("alias"),
        Some(&"new_alias".to_string())
    );
    assert_eq!(
        get_parser()
            .get_matches_from(vec!["-d", "anything"])
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

// #[test]
// fn test_handler() {
//     let arg_matches = get_argument_parser().get_matches_from(vec!["--", "echo", "TEST"]);
//     let result = handler().unwrap();
//     assert_eq!(result.stdout, b"TEST\n");
// }
