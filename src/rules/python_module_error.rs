use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

const MISSING_MODULE: &str = r"ModuleNotFoundError: No module named '([^']+)'";

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(output) = &command.output {
        output.contains("ModuleNotFoundError: No module named '")
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let missing_module = Regex::new(MISSING_MODULE)
            .unwrap()
            .captures(output)
            .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
            .unwrap_or("".to_owned());
        vec![system_shell.unwrap().and(vec![
            &format!("pip install {}", missing_module),
            &command.script,
        ])]
    } else {
        vec![]
    }
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

    fn module_error_output(filename: &str, module_name: &str) -> String {
        format!("Traceback (most recent call last):\n  File \"{}\", line 1, in <module>\n    import {}\nModuleNotFoundError: No module named '{}'", filename, module_name, module_name)
    }

    #[rstest]
    #[case("./some_other_script.py", &module_error_output("some_other_script.py", "a_module"), true)]
    #[case("python some_script.py", &module_error_output("some_script.py", "more_itertools"), true)]
    #[case("python hello_world.py", "Hello World", false)]
    #[case("./hello_world.py", "Traceback (most recent call last):\n  File \"hello_world.py\", line 1, in <module>\n    pritn(\"Hello World\")\nNameError: name 'pritn' is not defined", false)]
    fn test_not_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("./some_other_script.py", &module_error_output("some_other_script.py", "a_module"), vec!["pip install a_module && ./some_other_script.py"])]
    #[case("python some_script.py", &module_error_output("some_script.py", "more_itertools"), vec!["pip install more_itertools && python some_script.py"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_string()), None);
        assert_eq!(get_new_command(&mut command, Some(&system_shell)), expected);
    }
}
