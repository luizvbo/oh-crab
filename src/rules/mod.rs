use crate::shell::Shell;
use core::fmt;

use crate::{
    cli::{
        command::CorrectedCommand,
        command::{self, CrabCommand},
    },
    ui::interactive_menu,
};

mod apt_get;
pub mod apt_upgrade;
pub mod cargo;
pub mod no_command;

pub struct Rule {
    name: String,
    enabled_by_default: bool,
    priority: u16,
    requires_output: bool,
    pub match_rule: fn(&mut CrabCommand, Option<&Box<dyn Shell>>) -> bool,
    get_new_command: fn(&CrabCommand) -> Vec<String>,
    side_effect: Option<fn(CrabCommand, &String)>,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Rule {
    fn new(
        name: String,
        enabled_by_default: Option<bool>,
        priority: Option<u16>,
        requires_output: Option<bool>,
        match_rule: fn(&mut CrabCommand, Option<&Box<dyn Shell>>) -> bool,
        get_new_command: fn(&CrabCommand) -> Vec<String>,
        side_effect: Option<fn(CrabCommand, &String)>,
    ) -> Self {
        Self {
            name,
            enabled_by_default: enabled_by_default.unwrap_or(true),
            priority: priority.unwrap_or(1000),
            requires_output: requires_output.unwrap_or(true),
            match_rule,
            get_new_command,
            side_effect,
        }
    }

    // Returns `True` if rule matches the command.
    fn is_match(&self, mut command: CrabCommand, system_shell: &Box<dyn Shell>) -> bool {
        let script_only = command.stdout.is_none() && command.stderr.is_none();
        if script_only && self.requires_output {
            return false;
        }
        if (self.match_rule)(&mut command, Some(system_shell)) {
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

pub fn match_without_sudo(
    match_function: fn(&CrabCommand) -> bool,
    command: &mut CrabCommand,
) -> bool {
    if !command.script.starts_with("sudo ") {
        match_function(command)
    } else {
        let new_script = command.script[5..].to_owned();
        command.script = new_script;
        match_function(command)
    }
}

/// Generate a list of corrected commands for the given CrabCommand.
///
/// This function takes a `CrabCommand` as input and iterates through the registered
/// rules, applying each rule's match condition. The list of matching commands is then
/// reorganized and returned.
///
/// * `command`: A `CrabCommand` for which to generate corrected commands.
///
/// # Returns
///
/// A `Vec<CorrectedCommand>` containing the list of corrected commands based on the
/// input `CrabCommand`.
pub fn get_corrected_commands(
    command: &mut CrabCommand,
    system_shell: &Box<dyn Shell>,
) -> Vec<CorrectedCommand> {
    let mut corrected_commands: Vec<CorrectedCommand> = vec![];
    for rule in get_rules() {
        if (rule.match_rule)(command, Some(system_shell)) {
            for corrected in rule.get_corrected_commands(command) {
                corrected_commands.push(corrected);
            }
        }
    }
    organize_commands(corrected_commands)
}

pub fn organize_commands(mut corrected_commands: Vec<CorrectedCommand>) -> Vec<CorrectedCommand> {
    corrected_commands.sort_by(|a, b| a.priority.cmp(&b.priority));
    corrected_commands.dedup_by(|a, b| a.script.eq(&b.script));
    corrected_commands
}

pub fn selected_command(corrected_commands: &Vec<CorrectedCommand>) -> Option<&CorrectedCommand> {
    interactive_menu(corrected_commands)
}

pub fn get_rules() -> Vec<Rule> {
    vec![
        cargo::get_rule(),
        no_command::get_rule(),
        apt_upgrade::get_rule(),
    ]
}
