use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

use which::which;

use super::Rule;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    let stdout = command.output.clone().unwrap_or("".to_owned());
    if command.script_parts.len() != 1
        || which(&command.script_parts[0]).is_ok()
        || !(stdout.contains("No such file or directory")
            || stdout.contains("not found")
            || stdout.contains("is not recognised as"))
    {
        false
    } else {
        let re =
            Regex::new(r"((git|ssh|http(s)?)|(git@[\w\.]+))(:(//)?)([\w\.@\:/\-~]+)(\.git)?(/)?")
                .unwrap();
        re.is_match(&command.script)
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![format!("git clone {}", command.script)]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "git_clone_missing".to_owned(),
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

    parameterized_match_rule_tests! {
        match_rule,
        match_rule_01: ("https://github.com/nvbn/thefuck.git", "No such file or directory", true),
        match_rule_02: ("https://github.com/nvbn/thefuck.git", "not found", true),
        match_rule_03: ("https://github.com/nvbn/thefuck.git", "is not recognised as", true),
        match_rule_04: ("https://github.com/nvbn/thefuck", "No such file or directory", true),
        match_rule_05: ("https://github.com/nvbn/thefuck", "not found", true),
        match_rule_06: ("https://github.com/nvbn/thefuck", "is not recognised as", true),
        match_rule_07: ("http://github.com/nvbn/thefuck.git", "No such file or directory", true),
        match_rule_08: ("http://github.com/nvbn/thefuck.git", "not found", true),
        match_rule_09: ("http://github.com/nvbn/thefuck.git", "is not recognised as", true),
        match_rule_10: ("git@github.com:nvbn/thefuck.git", "No such file or directory", true),
        match_rule_11: ("git@github.com:nvbn/thefuck.git", "not found", true),
        match_rule_12: ("git@github.com:nvbn/thefuck.git", "is not recognised as", true),
        match_rule_13: ("git@github.com:nvbn/thefuck", "No such file or directory", true),
        match_rule_14: ("git@github.com:nvbn/thefuck", "not found", true),
        match_rule_15: ("git@github.com:nvbn/thefuck", "is not recognised as", true),
        match_rule_16: ("ssh://git@github.com:nvbn/thefuck.git", "No such file or directory", true),
        match_rule_17: ("ssh://git@github.com:nvbn/thefuck.git", "not found", true),
        unmatch_rule_01: ("", "No such file or directory", false),
        unmatch_rule_02: ("", "not found", false),
        unmatch_rule_03: ("", "is not recognised as", false),
        unmatch_rule_04: ("", "some other output", false),
        unmatch_rule_05: ("notacommand", "No such file or directory", false),
        unmatch_rule_06: ("notacommand", "not found", false),
        unmatch_rule_07: ("notacommand", "is not recognised as", false),
        unmatch_rule_08: ("notacommand", "some other output", false),
        unmatch_rule_09: ("ssh git@github.com:nvbn/thefrick.git", "No such file or directory", false),
        unmatch_rule_10: ("ssh git@github.com:nvbn/thefrick.git", "not found", false),
        unmatch_rule_11: ("ssh git@github.com:nvbn/thefrick.git", "is not recognised as", false),
        unmatch_rule_12: ("ssh git@github.com:nvbn/thefrick.git", "some other output", false),
        unmatch_rule_13: ("git clone foo", "No such file or directory", false),
        unmatch_rule_14: ("git clone foo", "not found", false),
        unmatch_rule_15: ("git clone foo", "is not recognised as", false),
        unmatch_rule_16: ("git clone foo", "some other output", false),
        unmatch_rule_17: ("git clone https://github.com/nvbn/thefuck.git", "No such file or directory", false),
        unmatch_rule_18: ("git clone https://github.com/nvbn/thefuck.git", "not found", false),
        unmatch_rule_19: ("git clone https://github.com/nvbn/thefuck.git", "is not recognised as", false),
        unmatch_rule_20: ("git clone https://github.com/nvbn/thefuck.git", "some other output", false),
        unmatch_rule_21: ("github.com/nvbn/thefuck.git", "No such file or directory", false),
        unmatch_rule_22: ("github.com/nvbn/thefuck.git", "not found", false),
        unmatch_rule_23: ("github.com/nvbn/thefuck.git", "is not recognised as", false),
        unmatch_rule_24: ("github.com/nvbn/thefuck.git", "some other output", false),
        unmatch_rule_25: ("github.com:nvbn/thefuck.git", "No such file or directory", false),
        unmatch_rule_26: ("github.com:nvbn/thefuck.git", "not found", false),
        unmatch_rule_27: ("github.com:nvbn/thefuck.git", "is not recognised as", false),
        unmatch_rule_28: ("github.com:nvbn/thefuck.git", "some other output", false),
        unmatch_rule_29: ("git clone git clone ssh://git@github.com:nvbn/thefrick.git", "No such file or directory", false),
        unmatch_rule_30: ("git clone git clone ssh://git@github.com:nvbn/thefrick.git", "not found", false),
        unmatch_rule_31: ("git clone git clone ssh://git@github.com:nvbn/thefrick.git", "is not recognised as", false),
        unmatch_rule_32: ("git clone git clone ssh://git@github.com:nvbn/thefrick.git", "some other output", false),
    }

    parameterized_get_new_command_tests! {
        get_new_command,
        get_new_command_01: ("https://github.com/nvbn/thefuck.git", "No such file or directory", "git clone https://github.com/nvbn/thefuck.git"),
        get_new_command_02: ("https://github.com/nvbn/thefuck.git", "not found", "git clone https://github.com/nvbn/thefuck.git"),
        get_new_command_03: ("https://github.com/nvbn/thefuck.git", "is not recognised as", "git clone https://github.com/nvbn/thefuck.git"),
        get_new_command_04: ("https://github.com/nvbn/thefuck", "No such file or directory", "git clone https://github.com/nvbn/thefuck"),
        get_new_command_05: ("https://github.com/nvbn/thefuck", "not found", "git clone https://github.com/nvbn/thefuck"),
        get_new_command_06: ("https://github.com/nvbn/thefuck", "is not recognised as", "git clone https://github.com/nvbn/thefuck"),
        get_new_command_07: ("http://github.com/nvbn/thefuck.git", "No such file or directory", "git clone http://github.com/nvbn/thefuck.git"),
        get_new_command_08: ("http://github.com/nvbn/thefuck.git", "not found", "git clone http://github.com/nvbn/thefuck.git"),
        get_new_command_09: ("http://github.com/nvbn/thefuck.git", "is not recognised as", "git clone http://github.com/nvbn/thefuck.git"),
        get_new_command_10: ("git@github.com:nvbn/thefuck.git", "No such file or directory", "git clone git@github.com:nvbn/thefuck.git"),
        get_new_command_11: ("git@github.com:nvbn/thefuck.git", "not found", "git clone git@github.com:nvbn/thefuck.git"),
        get_new_command_12: ("git@github.com:nvbn/thefuck.git", "is not recognised as", "git clone git@github.com:nvbn/thefuck.git"),
        get_new_command_13: ("git@github.com:nvbn/thefuck", "No such file or directory", "git clone git@github.com:nvbn/thefuck"),
        get_new_command_14: ("git@github.com:nvbn/thefuck", "not found", "git clone git@github.com:nvbn/thefuck"),
        get_new_command_15: ("git@github.com:nvbn/thefuck", "is not recognised as", "git clone git@github.com:nvbn/thefuck"),
        get_new_command_16: ("ssh://git@github.com:nvbn/thefuck.git", "No such file or directory", "git clone ssh://git@github.com:nvbn/thefuck.git"),
        get_new_command_17: ("ssh://git@github.com:nvbn/thefuck.git", "not found", "git clone ssh://git@github.com:nvbn/thefuck.git"),
        get_new_command_18: ("ssh://git@github.com:nvbn/thefuck.git", "is not recognised as", "git clone ssh://git@github.com:nvbn/thefuck.git"),
    }
}
