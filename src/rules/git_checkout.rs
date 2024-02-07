use super::{utils::git::get_command_with_git_support, Rule};
use crate::{
    cli::command::CrabCommand,
    rules::utils::{common::replace_argument, git::match_rule_with_git_support},
    shell::Shell,
    utils::get_closest,
};
use regex::Regex;

use std::str;
use std::process::Command;

fn get_branches(mock_output: Option<&str>) -> Vec<String> {
    let mut stdout: String;
    if let Some(mock_output_str) = mock_output {
        stdout = mock_output_str.to_owned();
    } else {
        let output = Command::new("git")
            .args(&["branch", "-a", "--no-color", "--no-column"])
            .output()
            .expect("Failed to execute command");
        stdout = str::from_utf8(&output.stdout).unwrap().to_owned();
    }

    let mut branches = Vec::new();

    for line in stdout.lines() {
        if line.contains("->") {
            // Remote HEAD like '  remotes/origin/HEAD -> origin/master'
            continue;
        }
        let mut line = line.to_string();
        if line.starts_with('*') {
            line = line.split_whitespace().nth(1).unwrap().to_string();
        }
        if line.trim().starts_with("remotes/") {
            line = line.split("/").skip(2).collect::<Vec<&str>>().join("/");
        }
        branches.push(line.trim().to_string());
    }

    branches
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        stdout.contains("did not match any file(s) known to git")
            && !stdout.contains("Did you forget to 'git add'?")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn mockable_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
    mock_output: Option<&str>,
) -> Vec<String> {
    if let Some(stdout) = &command.stdout {
        let re = Regex::new(r"error: pathspec '([^']*)' did not match any file\(s\) known to git")
            .unwrap();
        if let Some(caps) = re.captures(stdout) {
            let missing_file = caps.get(1).map_or("", |m| m.as_str());

            let branches = get_branches(mock_output);
            let branches: Vec<&str> = branches.iter().map(|s| s.as_str()).collect();
            let closest_branch = get_closest(missing_file, &branches, None, false);

            let mut new_commands = Vec::new();

            if !closest_branch.is_empty() {
                new_commands.push(replace_argument(
                    &command.script,
                    missing_file,
                    &closest_branch,
                ));
            }
            if command.script_parts.len() > 1 && command.script_parts[1] == "checkout" {
                new_commands.push(replace_argument(&command.script, "checkout", "checkout -b"));
            }

            if new_commands.is_empty() {
                new_commands.push(system_shell.unwrap().and(vec![
                    &format!("git branch {}", missing_file),
                    &command.script,
                ]));
            }
            new_commands
        } else {
            Vec::<String>::new()
        }
    } else {
        Vec::<String>::new()
    }
}


fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    mockable_get_new_command(command, system_shell, None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_checkout".to_owned(),
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
    use super::{mockable_get_new_command, match_rule};
    use crate::cli::command::CrabCommand;
    use crate::shell::Bash;

    use rstest::rstest;
    use std::io::Cursor;
    use std::process::Command;
    use std::str;

    fn did_not_match(target: &str, did_you_forget: bool) -> String {
        let mut error = format!(
            "error: pathspec '{}' did not match any file(s) known to git.",
            target
        );
        if did_you_forget {
            error = format!("{}\nDid you forget to 'git add'?'", error);
        }
        error
    }

    #[rstest]
    #[case("git checkout unknown", did_not_match("unknown", false))]
    #[case("git commit unknown", did_not_match("unknown", false))]
    fn test_match(#[case] command: &str, #[case] output: String) {
        let crab_command = &mut CrabCommand::new(command.to_owned(), Some(output), None);
        assert!(match_rule(crab_command, None));
    }

    // #[rstest]
    // #[case("git submodule update unknown", did_not_match("unknown", true))]
    // #[case("git checkout known", "")]
    // #[case("git commit known", "")]
    // fn test_not_match(#[case] command: &str, #[case] output: String) {
    //     let output = Command::new(command)
    //         .output()
    //         .expect("failed to execute process");
    //
    //     assert_ne!(str::from_utf8(&output.stdout).unwrap(), output);
    // }
    //
    // ... continue with the rest of the functions
}
