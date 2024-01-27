#[macro_export]
macro_rules! parameterized_match_rule_tests {
    ($match_rule:expr, $($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (script, stdout, check_value) = $value;
                let system_shell = Bash{};
                let mut command = CrabCommand::new(
                            script.to_owned(),
                            Some(stdout.to_owned()),
                            None
                        );
                assert_eq!($match_rule(&mut command, Some(&system_shell)), check_value);
            }
        )*
    }
}

#[macro_export]
macro_rules! parameterized_get_new_command_tests {
    ($get_new_command:expr, $($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (script, stdout, expected) = $value;
                let system_shell = Bash{};
                let mut command = CrabCommand::new(
                            script.to_owned(),
                            Some(stdout.to_owned()),
                            None
                        );
                assert_eq!($get_new_command(&mut command, Some(&system_shell))[0], expected);
            }
        )*
    }
}
