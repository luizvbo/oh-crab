use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell, utils::replace_command};
use regex::Regex;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(output) = &command.stdout {
        if output.contains("ambiguous command:") && output.contains("could be:") {
            return true;
        }
        return false;
    }
    false
}

pub fn get_new_command(
    command: &CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String> {

    let re = Regex::new(r"ambiguous command: (.*), could be: (.*)").unwrap();
    if let Some(stdout) = command.stdout {

        let caps = re.captures(&stdout).unwrap();

        let old_cmd = caps.get(1).unwrap().as_str();
        let suggestions: Vec<&str> = caps.get(2).unwrap().as_str().split(',').map(|s| s.trim()).collect();

        replace_command(command, old_cmd, suggestions)
    }
    else{
        Vec::<String>::new();
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


// @for_app('tmux')
// def match(command):
//     return ('ambiguous command:' in command.output
//             and 'could be:' in command.output)
//
//
// def get_new_command(command):
//     cmd = re.match(r"ambiguous command: (.*), could be: (.*)",
//                    command.output)
//
//     old_cmd = cmd.group(1)
//     suggestions = [c.strip() for c in cmd.group(2).split(',')]
//
//     return replace_command(command, old_cmd, suggestions)
