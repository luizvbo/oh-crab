pub mod git;


#[macro_export]
macro_rules! parameterized_match_rule_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (script, stdout, check_value) = $value;
                let mut command = CrabCommand::new(
                            script.to_owned(),
                            Some(stdout.to_owned()),
                            None
                        );
                assert_eq!(match_rule(&mut command, None), check_value);
            }
        )*
    }
}
