use crate::shell::Shell;
use core::fmt;

use crate::cli::{command::CorrectedCommand, command::CrabCommand};

mod ag_literal;
mod apt_get;
mod apt_get_search;
mod apt_list_upgradable;
mod apt_upgrade;
mod aws_cli;
mod az_cli;
mod brew_install;
mod brew_link;
mod brew_reinstall;
mod brew_uninstall;
mod brew_update_formula;
mod cargo;
mod cargo_no_command;
mod cat_dir;
mod cd_correction;
mod cd_cs;
mod cd_mkdir;
mod cd_parent;
mod chmod_x;
mod choco_install;
mod composer_not_command;
mod conda_mistype;
mod cp_create_destination;
mod cp_omitting_directory;
mod cpp11;
mod django_south_ghost;
mod django_south_merge;
mod docker_image_being_used_by_container;
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
mod git_hook_bypass;
mod git_main_master;
mod git_merge;
mod git_not_command;
mod git_pull;
mod git_pull_clone;
mod git_pull_uncommitted_changes;
mod git_push;
mod git_push_different_branch_names;
mod git_push_force;
mod git_push_pull;
mod git_push_without_commits;
mod git_rebase_merge_dir;
mod git_rebase_no_changes;
mod git_remote_delete;
mod go_run;
mod gradle_wrapper;
mod grep_arguments_order;
mod grep_recursive;
mod has_exists_script;
mod heroku_multiple_apps;
mod heroku_not_command;
mod history;
mod hostscli;
mod java;
mod javac;
mod lein_not_task;
mod ln_no_hard_link;
mod ln_s_order;
mod long_form_help;
mod ls_all;
mod ls_lah;
mod man;
mod man_no_space;
mod mercurial;
mod mkdir_p;
mod mvn_no_command;
mod mvn_unknown_lifecycle_phase;
mod nixos_cmd_not_found;
mod no_command;
mod no_such_file;
mod npm_missing_script;
mod npm_run_script;
mod php_s;
mod pip_install;
mod pip_unknown_command;
mod prove_recursively;
mod python_command;
mod python_execute;
mod python_module_error;
mod quotation_marks;
mod rails_migrations_pending;
mod remove_shell_prompt_literal;
mod rm_dir;
mod sudo;
mod sudo_command_from_user_path;
mod tmux;
mod unsudo;

mod utils;

pub fn get_rules() -> Vec<Rule> {
    vec![
        ag_literal::get_rule(),
        apt_get::get_rule(),
        apt_get_search::get_rule(),
        apt_list_upgradable::get_rule(),
        apt_upgrade::get_rule(),
        aws_cli::get_rule(),
        az_cli::get_rule(),
        brew_install::get_rule(),
        brew_link::get_rule(),
        brew_reinstall::get_rule(),
        brew_uninstall::get_rule(),
        brew_update_formula::get_rule(),
        cargo::get_rule(),
        cargo_no_command::get_rule(),
        cat_dir::get_rule(),
        cd_correction::get_rule(),
        cd_cs::get_rule(),
        cd_mkdir::get_rule(),
        cd_parent::get_rule(),
        chmod_x::get_rule(),
        choco_install::get_rule(),
        composer_not_command::get_rule(),
        conda_mistype::get_rule(),
        cp_create_destination::get_rule(),
        cp_omitting_directory::get_rule(),
        cpp11::get_rule(),
        django_south_ghost::get_rule(),
        django_south_merge::get_rule(),
        docker_image_being_used_by_container::get_rule(),
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
        git_push_pull::get_rule(),
        git_commit_reset::get_rule(),
        git_hook_bypass::get_rule(),
        git_diff_no_index::get_rule(),
        git_diff_staged::get_rule(),
        git_fix_stash::get_rule(),
        git_help_aliased::get_rule(),
        git_main_master::get_rule(),
        git_merge::get_rule(),
        git_not_command::get_rule(),
        git_pull::get_rule(),
        git_pull_clone::get_rule(),
        git_pull_uncommitted_changes::get_rule(),
        git_push::get_rule(),
        git_push_different_branch_names::get_rule(),
        git_push_force::get_rule(),
        git_push_without_commits::get_rule(),
        git_rebase_merge_dir::get_rule(),
        git_rebase_no_changes::get_rule(),
        git_remote_delete::get_rule(),
        go_run::get_rule(),
        gradle_wrapper::get_rule(),
        grep_arguments_order::get_rule(),
        grep_recursive::get_rule(),
        has_exists_script::get_rule(),
        heroku_multiple_apps::get_rule(),
        heroku_not_command::get_rule(),
        history::get_rule(),
        hostscli::get_rule(),
        java::get_rule(),
        javac::get_rule(),
        lein_not_task::get_rule(),
        ln_no_hard_link::get_rule(),
        ln_s_order::get_rule(),
        long_form_help::get_rule(),
        ls_all::get_rule(),
        ls_lah::get_rule(),
        man::get_rule(),
        man_no_space::get_rule(),
        mercurial::get_rule(),
        mkdir_p::get_rule(),
        mvn_no_command::get_rule(),
        mvn_unknown_lifecycle_phase::get_rule(),
        nixos_cmd_not_found::get_rule(),
        no_command::get_rule(),
        no_such_file::get_rule(),
        npm_missing_script::get_rule(),
        npm_run_script::get_rule(),
        php_s::get_rule(),
        pip_install::get_rule(),
        pip_unknown_command::get_rule(),
        prove_recursively::get_rule(),
        python_command::get_rule(),
        python_execute::get_rule(),
        python_module_error::get_rule(),
        quotation_marks::get_rule(),
        rails_migrations_pending::get_rule(),
        remove_shell_prompt_literal::get_rule(),
        rm_dir::get_rule(),
        sudo::get_rule(),
        sudo_command_from_user_path::get_rule(),
        tmux::get_rule(),
        unsudo::get_rule(),
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

pub fn match_rule_without_sudo<F>(match_function: F, command: &mut CrabCommand) -> bool
where
    F: Fn(&CrabCommand) -> bool,
{
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
