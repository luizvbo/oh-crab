use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell, utils::replace_command};
use regex::Regex;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(output) = &command.output {
        if output.contains("ambiguous command:") && output.contains("could be:") {
            return true;
        }
        return false;
    }
    false
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let re = Regex::new(r"ambiguous command: (.*), could be: (.*)").unwrap();
    if let Some(stdout) = &command.output {
        let caps = re.captures(stdout).unwrap();

        let old_cmd = caps.get(1).unwrap().as_str();
        let suggestions: Vec<&str> = caps
            .get(2)
            .unwrap()
            .as_str()
            .split(',')
            .map(|s| s.trim())
            .collect();

        replace_command(command, old_cmd, suggestions)
    } else {
        Vec::<String>::new()
    }
}
pub fn get_rule() -> Rule {
    Rule::new(
        "tmux".to_owned(),
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
    use crate::cli::command::CrabCommand;

    use super::{get_new_command, match_rule};

    const TMUX_AMBIGUOUS: &str = "ambiguous command: list, could be: list-buffers, list-clients, list-commands, list-keys, list-panes, list-sessions, list-windows";

    #[test]
    fn test_get_new_command() {
        let command = CrabCommand::new(
            "tmux list".to_owned(),
            Some(TMUX_AMBIGUOUS.to_owned()),
            None,
        );
        assert_eq!(
            get_new_command(&mut CrabCommand::new("tmux list".to_owned(), Some("ambiguous command: list, could be: list-buffers, list-clients, list-commands, list-keys, list-panes, list-sessions, list-windows".to_owned()), None), None),
            vec!["tmux list-keys", "tmux list-panes", "tmux list-buffers"]
        );
    }

    #[test]
    fn test_match() {
        let mut command = CrabCommand::new(
            "tmux list".to_owned(),
            Some(TMUX_AMBIGUOUS.to_owned()),
            None,
        );
        match_rule(&mut command, None);
    }
}
