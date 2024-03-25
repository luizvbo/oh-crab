use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell, utils::get_closest};
use regex::Regex;

fn extract_possibilities(command_output: &str) -> Vec<String> {
    let re1 = Regex::new(r"\n\(did you mean one of ([^\?]+)\?\)").unwrap();
    let re2 = Regex::new(r"\n    ([^$]+)$").unwrap();
    if let Some(caps) = re1.captures(command_output) {
        caps[1]
            .split(", ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else if let Some(caps) = re2.captures(command_output) {
        caps[1]
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else {
        vec![]
    }
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("hg: unknown command") && output.contains("(did you mean one of ")
            || output.contains("hg: command '") && output.contains("' is ambiguous:")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["hg"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let mut script = command.script_parts.clone();
        let possibilities = extract_possibilities(output);
        // script[1]
        if let Some(closest) = get_closest(
            &script[1],
            &possibilities.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
            None,
            None,
        ) {
            script[1] = closest.to_string();
            vec![script.join(" ")]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "mercurial".to_owned(),
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
    use super::{extract_possibilities, get_new_command, match_rule};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;
    use rstest::rstest;

    #[rstest]
    #[case(
        "hg branchch",
        "hg: unknown command 'branchch'\n(did you mean one of branch, branches?)",
        true
    )]
    #[case(
        "hg vert",
        "hg: unknown command 'vert'\n(did you mean one of revert?)",
        true
    )]
    #[case(
        "hg lgo -r tip",
        "hg: command 're' is ambiguous:\n(did you mean one of log?)",
        true
    )]
    #[case(
        "hg rerere",
        "hg: unknown command 'rerere'\n(did you mean one of revert?)",
        true
    )]
    #[case(
        "hg re",
        "hg: command 're' is ambiguous:\n    rebase recover remove rename resolve revert",
        true
    )]
    #[case(
        "hg re re",
        "hg: command 're' is ambiguous:\n    rebase recover remove rename resolve revert",
        true
    )]
    #[case("hg", "\nMercurial Distributed SCM\n\nbasic commands:", false)]
    #[case(
        "hg asdf",
        "hg: unknown command 'asdf'\nMercurial Distributed SCM\n\nbasic commands:",
        false
    )]
    #[case(
        "hg qwer",
        "hg: unknown command 'qwer'\nMercurial Distributed SCM\n\nbasic commands:",
        false
    )]
    #[case(
        "hg me",
        "\nabort: no repository found in './thefuck' (.hg not found)!",
        false
    )]
    #[case(
        "hg reb",
        "\nabort: no repository found in './thefuck' (.hg not found)!",
        false
    )]
    #[case(
        "hg co",
        "\nabort: no repository found in './thefuck' (.hg not found)!",
        false
    )]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("hg: unknown command 'base'\n(did you mean one of blame, phase, rebase?)", vec!["blame", "phase", "rebase"])]
    #[case("hg: unknown command 'branchch'\n(did you mean one of branch, branches?)", vec!["branch", "branches"])]
    #[case("hg: unknown command 'vert'\n(did you mean one of revert?)", vec!["revert"])]
    #[case("hg: command 're' is ambiguous:\n(did you mean one of log?)", vec!["log"])]
    #[case("hg: unknown command 'rerere'\n(did you mean one of revert?)", vec!["revert"])]
    #[case("hg: command 're' is ambiguous:\n    rebase recover remove rename resolve revert", vec!["rebase", "recover", "remove", "rename", "resolve", "revert"])]
    #[case("hg: command 're' is ambiguous:\n    rebase recover remove rename resolve revert", vec!["rebase", "recover", "remove", "rename", "resolve", "revert"])]
    fn test_extract_possibilities(#[case] stdout: &str, #[case] possibilities: Vec<&str>) {
        assert_eq!(extract_possibilities(stdout), possibilities)
    }

    #[rstest]
    #[case("hg base", "hg: unknown command 'base'\n(did you mean one of blame, phase, rebase?)", vec!["hg rebase"])]
    #[case("hg branchch", "hg: unknown command 'branchch'\n(did you mean one of branch, branches?)", vec!["hg branch"])]
    #[case("hg vert", "hg: unknown command 'vert'\n(did you mean one of revert?)", vec!["hg revert"])]
    #[case("hg lgo -r tip", "hg: command 're' is ambiguous:\n(did you mean one of log?)", vec!["hg log -r tip"])]
    #[case("hg rerere", "hg: unknown command 'rerere'\n(did you mean one of revert?)", vec!["hg revert"])]
    #[case("hg re", "hg: command 're' is ambiguous:\n    rebase recover remove rename resolve revert", vec!["hg rebase"])]
    #[case("hg re re", "hg: command 're' is ambiguous:\n    rebase recover remove rename resolve revert", vec!["hg rebase re"])]
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
