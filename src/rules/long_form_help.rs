use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(output) = &command.output {
        let re =
            Regex::new(r"(?:Run|Try) '([^']+)'(?: or '[^']+')? for (?:details|more information).")
                .unwrap();
        re.is_match(output) || output.contains("--help")
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let re =
            Regex::new(r"(?:Run|Try) '([^']+)'(?: or '[^']+')? for (?:details|more information).")
                .unwrap();
        if let Some(caps) = re.captures(output) {
            vec![caps.get(1).map_or("", |m| m.as_str()).to_owned()]
        } else {
            vec![command.script.replace("-h", "--help")]
        }
    } else {
        Vec::<String>::new()
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "long_form_help".to_owned(),
        None,
        Some(5000),
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

    #[rstest]
    #[case("grep -h", "Try 'grep --help' for more information.", true)]
    #[case("", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("grep -h", "", vec!["grep --help"])]
    #[case("tar -h", "", vec!["tar --help"])]
    #[case("docker run -h", "", vec!["docker run --help"])]
    #[case("cut -h", "", vec!["cut --help"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
