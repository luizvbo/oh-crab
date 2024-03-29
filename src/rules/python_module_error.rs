use super::{
    get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule,
};
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

const MISSING_MODULE: &str = r"ModuleNotFoundError: No module named '([^']+)'";

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("ModuleNotFoundError: No module named '")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo(
        |command| match_rule_with_is_app(auxiliary_match_rule, command, vec![], None),
        command,
    )
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let missing_module = Regex::new(MISSING_MODULE)
        .unwrap()
        .captures(&command.output)
        .and_then(|caps| caps.get(0).map(|m| m.as_str().to_owned()))
        .unwrap_or("".to_owned());
    vec![format!(
        "pip install {} && {}",
        missing_module, command.script
    )]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "python_module_error".to_owned(),
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
    use crate::shell::Bash;
    use rstest::rstest;

    const ERROR_MODULE_NOT_FOUND: &str = "Traceback (most recent call last):\n  File \"{}\", line 1, in <module>\n    import {}\nModuleNotFoundError: No module named '{}'";

    #[rstest]
    #[case("python hello_world.py", "Hello World", false)]
    #[case("./hello_world.py", "Traceback (most recent call last):\n  File \"hello_world.py\", line 1, in <module>\n    pritn(\"Hello World\")\nNameError: name 'pritn' is not defined", false)]
    fn test_not_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("python some_script.py", "some_script.py", "more_itertools", vec!["pip install more_itertools && python some_script.py"])]
    #[case("./some_other_script.py", "some_other_script.py", "a_module", vec!["pip install a_module && ./some_other_script.py"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] filename: &str,
        #[case] module_name: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(
            command.to_owned(),
            Some(format!(
                ERROR_MODULE_NOT_FOUND,
                filename, module_name, module_name
            )),
            None,
        );
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
