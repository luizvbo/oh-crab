use crate::{
    cli::command::CrabCommand, rules::utils::git::match_rule_with_git_support, shell::Shell,
};

use super::{utils::git::get_command_with_git_support, Rule};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.stdout {
        command.script.contains("pull") && stdout.contains("set-upstream")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_git_support(auxiliary_match_rule, command)
}

fn auxiliary_get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {
    if let Some(stdout) = &command.stdout {
        let lines: Vec<&str> = stdout.lines().collect();
        let line = lines[lines.len() - 3].trim();

        let words: Vec<&str> = line.split_whitespace().collect();
        let branch = words.last().unwrap_or(&"");
        let set_upstream = line
            .replace("<remote>", "origin")
            .replace("<branch>", branch);
        vec![system_shell
            .unwrap()
            .and(vec![&set_upstream, &command.script])]
    } else {
        Vec::<String>::new()
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_command_with_git_support(auxiliary_get_new_command, command, system_shell)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_pull".to_owned(),
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

    const OUTPUT: &str = r#"There is no tracking information for the current branch.
Please specify which branch you want to merge with.
See git-pull(1) for details

    git pull <remote> <branch>

If you wish to set tracking information for this branch you can do so with:

    git branch --set-upstream-to=<remote>/<branch> master


"#;

    parameterized_match_rule_tests! {
        match_rule,
        match_rule_01: ("git pull", OUTPUT, true),
        unmatch_rule_01: ("git pull", "", false),
        unmatch_rule_02: ("ls", OUTPUT, false),
    }

    parameterized_get_new_command_tests! {
        get_new_command,
        get_new_command_1: ("git pull", OUTPUT, "git branch --set-upstream-to=origin/master master && git pull"),
    }
}
