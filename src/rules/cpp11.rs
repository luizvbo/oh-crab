use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains(
            "This file requires compiler and library support for the ISO C++ 2011 standard.",
        ) || output.contains("-Wc++11-extensions")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["g++", "clang++"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![command.script.clone() + " -std=c++11"]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "cpp11".to_owned(),
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
    use super::{get_new_command, match_rule};
    use crate::cli::command::CrabCommand;
    use rstest::rstest;

    #[rstest]
    #[case("g++ foo.cpp", "foo.cpp:1:1: error: This file requires compiler and library support for the ISO C++ 2011 standard. This support must be enabled with the -std=c++11 or -std=gnu++11 compiler options.", true)]
    #[case("clang++ bar.cpp", "bar.cpp:1:1: warning: using extended identifiers requires -std=c++11 or -std=gnu++11 [-Wc++11-extensions]", true)]
    #[case(
        "g++ baz.cpp",
        "baz.cpp:1:1: error: 'auto' type specifier is a C++11 extension",
        false
    )]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("g++ foo.cpp", "foo.cpp:1:1: error: This file requires compiler and library support for the ISO C++ 2011 standard. This support must be enabled with the -std=c++11 or -std=gnu++11 compiler options.", vec!["g++ foo.cpp -std=c++11"])]
    #[case("clang++ bar.cpp", "bar.cpp:1:1: warning: using extended identifiers requires -std=c++11 or -std=gnu++11 [-Wc++11-extensions]", vec!["clang++ bar.cpp -std=c++11"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
