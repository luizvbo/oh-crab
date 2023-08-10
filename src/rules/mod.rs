use crate::{cli::command::CrabCommand, command::CorrectedCommand};

mod apt_get;
pub mod cargo;

pub struct Rule {
    name: String,
    enabled_by_default: bool,
    priority: u16,
    requires_output: bool,
    pub match_rule: fn(&CrabCommand) -> bool,
    get_new_command: fn(&CrabCommand) -> Vec<String>,
    side_effect: Option<fn(CrabCommand, &String)>,
}

impl Rule {
    fn new(
        name: String,
        enabled_by_default: Option<bool>,
        priority: Option<u16>,
        requires_output: Option<bool>,
        match_rule: fn(&CrabCommand) -> bool,
        get_new_command: fn(&CrabCommand) -> Vec<String>,
        side_effect: Option<fn(CrabCommand, &String)>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            enabled_by_default: enabled_by_default.unwrap_or(true),
            priority: priority.unwrap_or(1000),
            requires_output: requires_output.unwrap_or(true),
            match_rule,
            get_new_command,
            side_effect,
        }
    }

    // Returns `True` if rule matches the command.
    fn is_match(&self, command: CrabCommand) -> bool {
        let script_only = command.stdout.is_none() && command.stderr.is_none();
        if script_only == true && self.requires_output == true {
            return false;
        }
        if (self.match_rule)(&command) {
            return true;
        }
        false
    }

    fn get_corrected_commands(&self, command: &CrabCommand) -> Vec<CorrectedCommand> {
        let mut new_commands: Vec<CorrectedCommand> = vec![];
        for (n, new_command) in (self.get_new_command)(command).iter().enumerate() {
            new_commands.push(CorrectedCommand::new(
                new_command.to_owned(),
                self.side_effect,
                (n as u16 + 1) * self.priority,
            ));
        }
        new_commands
    }
}

pub fn get_corrected_commands(command: CrabCommand) -> Vec<CorrectedCommand> {
    let mut corrected_commands: Vec<CorrectedCommand> = vec![];
    for rule in get_rules() {
        if (rule.match_rule)(&command) {
            for corrected in rule.get_corrected_commands(&command) {
                corrected_commands.push(corrected);
            }
        }
    }
    return organize_commands(corrected_commands);
}

pub fn organize_commands(mut corrected_commands: Vec<CorrectedCommand>) -> Vec<CorrectedCommand> {
    corrected_commands.sort_by(|a, b| a.priority.cmp(&b.priority));
    corrected_commands.dedup_by(|a, b| a.script.eq(&b.script));
    corrected_commands
}

pub fn selected_command(corrected_commands: Vec<CorrectedCommand>) -> CorrectedCommand {
    corrected_commands.into_iter().next().expect("")
}

pub fn get_rules() -> Vec<Rule> {
    vec![cargo::get_rule()]
}
