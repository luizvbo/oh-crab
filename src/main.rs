use clap::{command, error::ErrorKind, Arg, ArgAction, Command};
use std::env;

const ARGUMENT_PLACEHOLDER: &'static str = "OHCRAB_ARGUMENT_PLACEHOLDER";

fn main() {
    let args: Vec<String> = env::args().collect();

    // let args = read_arguments();
    // let s_args = args.ids().map(|id| id.as_str()).collect::<Vec<_>>();
    // println!("{:?}", s_args);
}

fn prepare_arguments(mut argv: Vec<String>) -> Vec<String> {
    match argv.iter().position(|x| x == ARGUMENT_PLACEHOLDER) {
        Some(index) => {
            let mut argv_processed = Vec::with_capacity(argv.len() + 1);
            argv_processed.extend_from_slice(&argv[index + 1..]);
            argv_processed.push("--".to_string());
            argv_processed.extend_from_slice(&argv[..index]);
            argv_processed
        }
        None => {
            if argv.len() > 0 && !argv[0].starts_with('-') && argv[0] != "--" {
                argv.insert(0, "--".to_string());
            }
            argv
        }
    }
}

#[test]
fn test_prepare_arguments() {
    assert_eq!(
        prepare_arguments(vec![
            "arg1".to_string(),
            "arg2".to_string(),
            "OHCRAB_ARGUMENT_PLACEHOLDER".to_string(),
            "arg3".to_string(),
            "arg4".to_string()
        ]),
        vec!["arg3", "arg4", "--", "arg1", "arg2"]
    );
}

/// Generate an argument parser using clap
fn get_argument_parser() -> Command {
    command!()
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
            Arg::new("yes")
                .long("yes")
                .short('y')
                .help("Execute fixed command without confirmation")
                .action(ArgAction::SetTrue),
        )
        // It's too dangerous to use `-y` and `-r` together.
        .arg(
            Arg::new("repeat")
                .long("repeat")
                .short('r')
                .help("Repeat on failure")
                .action(ArgAction::SetTrue)
                .conflicts_with("yes"),
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
            Arg::new("force-command")
                .required(false)
                .action(ArgAction::Set),
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
fn test_get_argument_parser() {
    env::set_var("OC_ALIAS", "env_alias");
    // Test alias defined from environment variable
    assert_eq!(
        get_argument_parser()
            .get_matches_from(vec!["ohcrab"])
            .get_one::<String>("alias"),
        Some(&"env_alias".to_string())
    );
    assert_eq!(
        get_argument_parser()
            .get_matches_from(vec!["ohcrab", "--alias", "new_alias"])
            .get_one::<String>("alias"),
        Some(&"new_alias".to_string())
    );
    assert_eq!(
        get_argument_parser()
            .get_matches_from(vec!["ohcrab", "-a", "new_alias"])
            .get_one::<String>("alias"),
        Some(&"new_alias".to_string())
    );
    assert_eq!(
        get_argument_parser()
            .get_matches_from(vec!["ohcrab", "-d", "anything"])
            .get_flag("debug"),
        true
    );
    assert_eq!(
        get_argument_parser()
            .get_matches_from(vec!["ohcrab", "--", "my", "command"])
            .get_many::<String>("command")
            .expect("Command not found")
            .map(|x| x.as_str())
            .collect::<Vec<_>>(),
        ["my", "command"]
    );
    // Test conflicting arguments
    assert_eq!(
        get_argument_parser()
            .try_get_matches_from(vec!["ohcrab", "-y", "-r"])
            .unwrap_err()
            .kind(),
        ErrorKind::ArgumentConflict
    );
    assert_eq!(
        get_argument_parser()
            .get_matches_from(vec!["ohcrab", "-r"])
            .get_flag("yes"),
        false
    );
    assert_eq!(
        get_argument_parser()
            .get_matches_from(vec!["ohcrab", "-y"])
            .get_flag("yes"),
        true
    );
    assert_eq!(
        get_argument_parser()
            .get_matches_from(vec!["ohcrab", "-r"])
            .get_flag("repeat"),
        true
    );
}
