use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn get_name(command_output: &str) -> Option<String> {
    let re = Regex::new(r"nix-env -iA ([^\s]*)").unwrap();
    re.captures(command_output)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(output) = &command.output {
        get_name(output).is_some()
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let name = get_name(output);
        if let Some(name) = name {
            vec![format!("nix-env -iA {} && {}", name, command.script)]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "nixos_cmd_not_found".to_owned(),
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

    #[rstest]
    #[case("vim", "nix-env -iA nixos.vim", true)]
    #[case("vim", "", false)]
    #[case("", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("vim", "nix-env -iA nixos.vim", vec!["nix-env -iA nixos.vim && vim"])]
    #[case("pacman", "nix-env -iA nixos.pacman", vec!["nix-env -iA nixos.pacman && pacman"])]
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
