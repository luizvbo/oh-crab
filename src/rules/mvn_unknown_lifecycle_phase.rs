use super::{ get_new_command_without_sudo, match_rule_without_sudo, utils::match_rule_with_is_app, Rule, };
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

fn get_failed_lifecycle(command: &CrabCommand) -> Option<String> {
    let re = Regex::new(r"\[ERROR\] Unknown lifecycle phase \"(.+)\"").unwrap();
    re.captures(&command.output).and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
}

fn getavailable_lifecycles(command: &CrabCommand) -> Option<Vec<String>> {
    let re = Regex::new(r"Available lifecycle phases are: (.+) -> \[Help 1\]").unwrap();
    re.captures(&command.output).and_then(|caps| caps.get(1).map(|m| m.as_str().split(", ").map(|s| s.to_owned()).collect()))
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        command.script_parts.first().map_or(false, |s| s == "mvn") && get_failed_lifecycle(command).is_some() && getavailable_lifecycles(command).is_some()
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_without_sudo( |command| match_rule_with_is_app(auxiliary_match_rule, command, vec!["mvn"], None), command,)
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let failed_lifecycle = get_failed_lifecycle(command);
    let available_lifecycles = getavailable_lifecycles(command);
    if let (Some(failed_lifecycle), Some(available_lifecycles)) = (failed_lifecycle, available_lifecycles) {
        let selected_lifecycle = available_lifecycles.iter().min_by_key(|a| levenshtein::levenshtein(a, &failed_lifecycle)).unwrap_or(&"".to_string()).to_owned();
        vec![command.script.replace(&failed_lifecycle, &selected_lifecycle)]
    } else {
        vec![]
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "mvn_unknown_lifecycle_phase".to_owned(),
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
    use rstest::rstest;

    const ERROR_UNKNOWN_LIFECYCLE: &str = "[ERROR] Unknown lifecycle phase \"cle\". You must specify a valid lifecycle phase or a goal in the format <plugin-prefix>:<goal> or <plugin-group-id>:<plugin-artifact-id>[:<plugin-version>]:<goal>. Available lifecycle phases are: validate, initialize, generate-sources, process-sources, generate-resources, process-resources, compile, process-classes, generate-test-sources, process-test-sources, generate-test-resources, process-test-resources, test-compile, process-test-classes, test, prepare-package, package, pre-integration-test, integration-test, post-integration-test, verify, install, deploy, pre-clean, clean, post-clean, pre-site, site, post-site, site-deploy. -> [Help 1]";
    const MVN_CLEAN: &str = "\n[INFO] Scanning for projects...[INFO]                                                                         
[INFO] ------------------------------------------------------------------------
[INFO] Building test 0.2
[INFO] ------------------------------------------------------------------------
[INFO] 
[INFO] --- maven-clean-plugin:2.5:clean (default-clean) @ test ---
[INFO] Deleting /home/mlk/code/test/target
[INFO] ------------------------------------------------------------------------
[INFO] BUILD SUCCESS
[INFO] ------------------------------------------------------------------------
[INFO] Total time: 0.477s
[INFO] Finished at: Wed Aug 26 13:05:47 BST 2015
[INFO] Final Memory: 6M/240M
[INFO] ------------------------------------------------------------------------";

    #[rstest]
    #[case("mvn cle", ERROR_UNKNOWN_LIFECYCLE, true)]
    #[case("mvn clean", MVN_CLEAN, false)]
    #[case("mvn --help", "", false)]
    #[case("mvn -v", "", false)]
    fn test_match
