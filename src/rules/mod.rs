use crate::shell::Shell;
use core::fmt;

use crate::cli::{command::CorrectedCommand, command::CrabCommand};

mod ag_literal;
mod apt_get;
mod apt_get_search;
mod apt_list_upgradable;
mod apt_upgrade;
mod aws_cli;
mod brew_install;
mod brew_update_formula;
mod cargo;
mod cat_dir;
mod cd_correction;
mod cd_cs;
mod cd_mkdir;
mod cd_parent;
mod chmod_x;
mod choco_install;
mod cp_create_destination;
mod cp_omitting_directory;
mod django_south_ghost;
mod docker_login;
mod dry;
mod fix_alt_space;
mod git_add;
mod git_add_force;
mod git_bisect_usage;
mod git_branch_0flag;
mod git_branch_delete;
mod git_branch_delete_checked_out;
mod git_branch_exists;
mod git_branch_list;
mod git_checkout;
mod git_clone;
mod git_clone_missing;
mod git_commit_add;
mod git_commit_amend;
mod git_commit_reset;
mod git_diff_no_index;
mod git_diff_staged;
mod git_fix_stash;
mod git_help_aliased;
mod git_main_master;
mod git_merge;
mod git_not_command;
mod git_pull;
mod git_push;
mod git_rebase_merge_dir;
mod go_run;
mod grep_arguments_order;
mod grep_recursive;
mod history;
mod java;
mod ln_no_hard_link;
mod ls_all;
mod ls_lah;
mod mkdir_p;
mod no_command;
mod python_command;
mod python_execute;
mod quotation_marks;
mod rm_dir;
mod tmux;

mod utils;

pub fn get_rules() -> Vec<Rule> {
    vec![
        ag_literal::get_rule(),
        apt_get::get_rule(),
        apt_get_search::get_rule(),
        apt_list_upgradable::get_rule(),
        apt_upgrade::get_rule(),
        aws_cli::get_rule(),
        brew_install::get_rule(),
        brew_update_formula::get_rule(),
        cargo::get_rule(),
        cat_dir::get_rule(),
        cd_correction::get_rule(),
        cd_cs::get_rule(),
        cd_mkdir::get_rule(),
        cd_parent::get_rule(),
        chmod_x::get_rule(),
        choco_install::get_rule(),
        cp_create_destination::get_rule(),
        cp_omitting_directory::get_rule(),
        django_south_ghost::get_rule(),
        docker_login::get_rule(),
        dry::get_rule(),
        fix_alt_space::get_rule(),
        git_add::get_rule(),
        git_add_force::get_rule(),
        git_bisect_usage::get_rule(),
        git_branch_0flag::get_rule(),
        git_branch_delete::get_rule(),
        git_branch_delete_checked_out::get_rule(),
        git_branch_exists::get_rule(),
        git_branch_list::get_rule(),
        git_checkout::get_rule(),
        git_clone::get_rule(),
        git_clone_missing::get_rule(),
        git_commit_add::get_rule(),
        git_commit_amend::get_rule(),
        git_commit_reset::get_rule(),
        git_diff_no_index::get_rule(),
        git_diff_staged::get_rule(),
        git_fix_stash::get_rule(),
        git_help_aliased::get_rule(),
        git_main_master::get_rule(),
        git_merge::get_rule(),
        git_not_command::get_rule(),
        git_pull::get_rule(),
        git_push::get_rule(),
        git_rebase_merge_dir::get_rule(),
        go_run::get_rule(),
        grep_arguments_order::get_rule(),
        grep_recursive::get_rule(),
        history::get_rule(),
        java::get_rule(),
        ln_no_hard_link::get_rule(),
        ls_all::get_rule(),
        ls_lah::get_rule(),
        mkdir_p::get_rule(),
        no_command::get_rule(),
        python_command::get_rule(),
        python_execute::get_rule(),
        quotation_marks::get_rule(),
        rm_dir::get_rule(),
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
        let script_only = command.output.is_none();
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

pub fn match_rule_without_sudo(
    match_function: fn(&CrabCommand) -> bool,
    command: &mut CrabCommand,
) -> bool {
    if !command.script.starts_with("sudo ") {
        match_function(command)
    } else {
        let new_script = command.script[5..].to_owned();
        match_function(&command.update(Some(new_script), None, None))
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
