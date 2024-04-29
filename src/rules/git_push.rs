use crate::{
    cli::command::CrabCommand,
    rules::utils::git::{get_new_command_with_git_support, match_rule_with_git_support},
    rules::Rule,
    shell::Shell,
    utils::replace_argument,
};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        command.script_parts.contains(&"push".to_owned())
            && stdout.contains("git push --set-upstream")
    } else {
        false
    }
}

fn get_upstream_option_index(command_parts: &[String]) -> Option<usize> {
    if command_parts.contains(&"--set-upstream".to_owned()) {
        command_parts.iter().position(|r| r == "--set-upstream")
    } else if command_parts.contains(&"-u".to_owned()) {
        command_parts.iter().position(|r| r == "-u")
    } else {
        None
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    if let Some(stdout) = &command.output {
        // If --set-upstream or -u are passed, remove it and its argument. This is
        // because the remaining arguments are concatenated onto the command suggested
        // by git, which includes --set-upstream and its argument
        let mut command_parts = command.script_parts.clone();
        let upstream_option_index = get_upstream_option_index(&command_parts);

        if let Some(index) = upstream_option_index {
            command_parts.remove(index);
            if command_parts.len() > index {
                command_parts.remove(index);
            }
        } else {
            let push_idx = command_parts.iter().position(|r| r == "push").unwrap() + 1;
            while command_parts.len() > push_idx
                && !command_parts[command_parts.len() - 1].starts_with('-')
            {
                command_parts.pop();
            }
        }

        let re = Regex::new(r"git push (.*)").unwrap();
        let arguments = re.captures(stdout).unwrap()[1]
            .replace('\'', r"\'")
            .trim()
            .to_string();
        vec![replace_argument(
            &command_parts.join(" "),
            "push",
            &format!("push {}", arguments),
        )]
    } else {
        Vec::<String>::new()
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_push".to_owned(),
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
    use crate::{parameterized_get_new_command_tests, parameterized_match_rule_tests};

    const OUTPUT_BITBUCKET: &str = "r#
Total 0 (delta 0), reused 0 (delta 0)
remote:
remote: Create pull request for feature/set-upstream:
remote:   https://bitbucket.org/set-upstream
remote:
To git@bitbucket.org:test.git
   e5e7fbb..700d998  feature/set-upstream -> feature/set-upstream
Branch feature/set-upstream set up to track remote branch feature/set-upstream from origin.
#";

    fn output_with_branch(branch_name: &str) -> String {
        if branch_name.is_empty() {
            "".to_string()
        } else {
            format!(
                "fatal: The current branch {} has no upstream branch.
To push the current branch and set the remote as upstream, use

    git push --set-upstream origin {}

",
                branch_name, branch_name
            )
        }
    }

    parameterized_match_rule_tests! {
        match_rule,
        match_rule_01: ("git push", output_with_branch("master"), true),
        match_rule_02: ("git push origin", output_with_branch("master"), true),
        unmatch_rule_01: ("git push origin", OUTPUT_BITBUCKET, false),
        unmatch_rule_02: ("git push master", output_with_branch(""), false),
        unmatch_rule_03: ("ls", output_with_branch("master"), false),
    }

    parameterized_get_new_command_tests! {
        get_new_command,
        get_new_command_1: ("git push", output_with_branch("master"), "git push --set-upstream origin master"),
        get_new_command_2: ("git push master", output_with_branch("master"), "git push --set-upstream origin master"),
        get_new_command_3: ("git push -u", output_with_branch("master"), "git push --set-upstream origin master"),
        get_new_command_4: ("git push -u origin", output_with_branch("master"), "git push --set-upstream origin master"),
        get_new_command_5: ("git push origin", output_with_branch("master"), "git push --set-upstream origin master"),
        get_new_command_6: ("git push --set-upstream origin", output_with_branch("master"), "git push --set-upstream origin master"),
        get_new_command_7: ("git push --quiet", output_with_branch("master"), "git push --set-upstream origin master --quiet"),
        get_new_command_8: ("git push --quiet origin", output_with_branch("master"), "git push --set-upstream origin master --quiet"),
        get_new_command_9: ("git -c test=test push --quiet origin", output_with_branch("master"), "git -c test=test push --set-upstream origin master --quiet"),
        get_new_command_10: ("git push", output_with_branch("test's"), "git push --set-upstream origin test\\'s"),
        get_new_command_11: ("git push --force", output_with_branch("master"), "git push --set-upstream origin master --force"),
        get_new_command_12: ("git push --force-with-lease", output_with_branch("master"), "git push --set-upstream origin master --force-with-lease"),
    }
}
