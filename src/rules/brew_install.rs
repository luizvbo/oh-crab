use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

use super::Rule;

fn get_suggestions(str: String) -> Vec<String> {
    str.replace(" or ", ", ")
        .split(", ")
        .map(|s| s.to_string())
        .collect()
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(stdout) = &command.output {
        command.script.contains("install")
            && stdout.contains("No available formula")
            && stdout.contains("Did you mean")
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let re = Regex::new(
        "Warning: No available formula with the name \"(?:[^\"]+)\". Did you mean (.+)\\?",
    )
    .unwrap();
    let stdout = &command.output.as_ref().unwrap();
    let caps = re.captures(stdout).unwrap();
    let suggestions = get_suggestions(caps.get(1).map_or("", |m| m.as_str()).to_owned());
    suggestions
        .iter()
        .map(|formula| format!("brew install {formula}"))
        .collect()
}

pub fn get_rule() -> Rule {
    Rule::new(
        "brew_install".to_owned(),
        None,
        None,
        None,
        match_rule,
        get_new_command,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::{get_new_command, get_suggestions, match_rule};
    use crate::cli::command::CrabCommand;

    const BREW_NO_AVAILABLE_FORMULA_ONE: &str =
        "Warning: No available formula with the name \"giss\". Did you mean gist?";
    const BREW_NO_AVAILABLE_FORMULA_TWO: &str = "Warning: No available formula with the name \"elasticserar\". Did you mean elasticsearch or elasticsearch@6?";
    const BREW_NO_AVAILABLE_FORMULA_THREE: &str =
        "Warning: No available formula with the name \"gitt\". Did you mean git, gitg or gist?";
    const BREW_INSTALL_NO_ARGUMENT: &str =
        "Install a formula or cask. Additional options specific to a formula may be";
    const BREW_ALREADY_INSTALLED: &str = "Warning: git-2.3.5 already installed";

    macro_rules! parameterized_match_rule_tests {
        ($match_rule:expr, $($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!($match_rule(&mut command, None));
                }
            )*
        }
    }

    macro_rules! parameterized_unmatch_rule_tests {
        ($match_rule:expr, $($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!(!$match_rule(&mut command, None));
                }
            )*
        }
    }

    macro_rules! parameterized_get_new_command_tests {
        ($get_new_command:expr, $($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout, expected, neq) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    if neq{
                        assert_ne!($get_new_command(&mut command, None), expected);
                    }
                    else{
                        assert_eq!($get_new_command(&mut command, None), expected);
                    }
                }
            )*
        }
    }

    parameterized_match_rule_tests! {
        match_rule,
        match_rule_1: ("brew install giss", BREW_NO_AVAILABLE_FORMULA_ONE),
        match_rule_2: ("brew install elasticserar", BREW_NO_AVAILABLE_FORMULA_TWO),
        match_rule_3: ("brew install gitt", BREW_NO_AVAILABLE_FORMULA_THREE),
    }

    parameterized_unmatch_rule_tests! {
        match_rule,
        unmatch_rule_1: ("brew install git", BREW_ALREADY_INSTALLED),
        unmatch_rule_2: ("brew install", BREW_INSTALL_NO_ARGUMENT),
    }

    parameterized_get_new_command_tests! {
        get_new_command,
        get_new_command_1: ("brew install giss", BREW_NO_AVAILABLE_FORMULA_ONE, ["brew install gist"], false),
        get_new_command_2: ("brew install elasticsear", BREW_NO_AVAILABLE_FORMULA_TWO, ["brew install elasticsearch", "brew install elasticsearch@6"], false),
        get_new_command_3: ("brew install gitt", BREW_NO_AVAILABLE_FORMULA_THREE, ["brew install git", "brew install gitg", "brew install gist"], false),
        get_new_command_4: ("brew install aa", BREW_NO_AVAILABLE_FORMULA_ONE, ["brew install aha"], true),
    }

    #[test]
    fn test_suggestions() {
        assert_eq!(get_suggestions("one".to_owned()), ["one"]);
        assert_eq!(get_suggestions("one or two".to_owned()), ["one", "two"]);
        assert_eq!(
            get_suggestions("one, two or three".to_owned()),
            ["one", "two", "three"]
        );
    }
}
