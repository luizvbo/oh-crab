use crate::shell::Shell;
use core::fmt;

use crate::{
    cli::{command::CorrectedCommand, command::CrabCommand},
    ui::interactive_menu,
};

mod ag_literal;
mod apt_get;
mod apt_get_search;
mod apt_list_upgradable;
mod apt_upgrade;
mod brew_install;
mod brew_update_formula;
mod cargo;
mod cd_correction;
mod cd_cs;
mod cd_mkdir;
mod cd_parent;
mod chmod_x;
mod choco_install;
mod git_add;
mod git_add_force;
mod git_bisect_usage;
mod git_branch_delete;
mod git_branch_list;
mod git_checkout;
mod git_clone;
mod git_clone_missing;
mod git_commit_add;
mod git_main_master;
mod git_merge;
mod git_not_command;
mod git_pull;
mod git_push;
mod history;
mod no_command;
mod tmux;

mod utils;

pub fn get_rules() -> Vec<Rule> {
    vec![
        apt_get::get_rule(),
        ag_literal::get_rule(),
        apt_get_search::get_rule(),
        apt_upgrade::get_rule(),
        apt_list_upgradable::get_rule(),
        brew_install::get_rule(),
        brew_update_formula::get_rule(),
        cargo::get_rule(),
        cd_correction::get_rule(),
        cd_cs::get_rule(),
        chmod_x::get_rule(),
        choco_install::get_rule(),
        cd_parent::get_rule(),
        cd_mkdir::get_rule(),
        git_add::get_rule(),
        git_add_force::get_rule(),
        git_branch_list::get_rule(),
        git_branch_delete::get_rule(),
        git_checkout::get_rule(),
        git_clone::get_rule(),
        git_clone_missing::get_rule(),
        git_commit_add::get_rule(),
        git_bisect_usage::get_rule(),
        git_main_master::get_rule(),
        git_merge::get_rule(),
        git_not_command::get_rule(),
        git_pull::get_rule(),
        git_push::get_rule(),
        history::get_rule(),
        no_command::get_rule(),
        tmux::get_rule(),
    ]
}
pub struct Rule {
    name: String,
    enabled_by_default: bool,
    priority: u16,
    requires_output: bool,
    pub match_rule: fn(&mut CrabCommand, Option<&dyn Shell>) -> bool,
    get_new_command: fn(&mut CrabCommand, Option<&dyn Shell>) -> Vec<String>,
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
        match_rule: fn(&mut CrabCommand, Option<&dyn Shell>) -> bool,
        get_new_command: fn(&mut CrabCommand, Option<&dyn Shell>) -> Vec<String>,
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
    fn is_match(&self, mut command: CrabCommand, system_shell: &dyn Shell) -> bool {
        let script_only = command.stdout.is_none() && command.stderr.is_none();
        if script_only && self.requires_output {
            return false;
        }
        if (self.match_rule)(&mut command, Some(system_shell)) {
            return true;
        }
        false
    }

    fn get_corrected_commands(
        &self,
        command: &mut CrabCommand,
        system_shell: &dyn Shell,
    ) -> Vec<CorrectedCommand> {
        let mut new_commands: Vec<CorrectedCommand> = vec![];
        for (n, new_command) in (self.get_new_command)(command, Some(system_shell))
            .iter()
            .enumerate()
        {
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

pub fn get_new_command_without_sudo(
    get_new_command_function: fn(&CrabCommand) -> Vec<String>,
    command: &mut CrabCommand,
) -> Vec<String> {
    if !command.script.starts_with("sudo ") {
        get_new_command_function(command)
    } else {
        let new_script = command.script[5..].to_owned();
        command.script = new_script;
        get_new_command_function(command)
            .iter()
            .map(|cmd| "sudo ".to_owned() + cmd)
            .collect()
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
    system_shell: &dyn Shell,
) -> Vec<CorrectedCommand> {
    let mut corrected_commands: Vec<CorrectedCommand> = vec![];
    for rule in get_rules() {
        if (rule.match_rule)(command, Some(system_shell)) {
            for corrected in rule.get_corrected_commands(command, system_shell) {
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

pub fn selected_command(corrected_commands: &[CorrectedCommand]) -> Option<&CorrectedCommand> {
    interactive_menu(corrected_commands)
}
