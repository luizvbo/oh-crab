use super::{utils::match_rule_with_is_app, Rule};
use crate::{
    cli::command::CrabCommand,
    shell::Shell,
    utils::{get_close_matches, replace_command},
};
use regex::Regex;

fn get_failed_lifecycle(command_output: &str) -> Option<String> {
    let re = Regex::new(r#"\[ERROR\] Unknown lifecycle phase \"(.+)\""#).unwrap();
    if let Some(caps) = re.captures(command_output) {
        Some(caps[1].to_string())
    } else {
        None
    }
}

fn getavailable_lifecycles(command_output: &str) -> Option<String> {
    let re = Regex::new(r"Available lifecycle phases are: (.+) -> \[Help 1\]").unwrap();
    if let Some(caps) = re.captures(command_output) {
        Some(caps[1].to_string())
    } else {
        None
    }
}

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        get_failed_lifecycle(output).is_some() && getavailable_lifecycles(output).is_some()
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["mvn"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let failed_lifecycle = get_failed_lifecycle(output);
        let available_lifecycles = getavailable_lifecycles(output);
        if let (Some(failed_lifecycle), Some(available_lifecycles)) =
            (failed_lifecycle, available_lifecycles)
        {
            let available_lifecycles: Vec<&str> = available_lifecycles.split(", ").collect();
            let selected_lifecycle =
                get_close_matches(&failed_lifecycle, &available_lifecycles, None, None);
            replace_command(command, &failed_lifecycle, selected_lifecycle)
        } else {
            vec![]
        }
    } else {
        vec![]
    }
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
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }
    #[rstest]
    #[case("mvn cle", r#"[ERROR] Unknown lifecycle phase "cle". You must specify a valid lifecycle phase or a goal in the format <plugin-prefix>:<goal> or <plugin-group-id>:<plugin-artifact-id>[:<plugin-version>]:<goal>. Available lifecycle phases are: validate, initialize, generate-sources, process-sources, generate-resources, process-resources, compile, process-classes, generate-test-sources, process-test-sources, generate-test-resources, process-test-resources, test-compile, process-test-classes, test, prepare-package, package, pre-integration-test, integration-test, post-integration-test, verify, install, deploy, pre-clean, clean, post-clean, pre-site, site, post-site, site-deploy. -> [Help 1]"#, vec!["mvn clean", "mvn compile"])]
    #[case("mvn claen package", r#"[ERROR] Unknown lifecycle phase "claen". You must specify a valid lifecycle phase or a goal in the format <plugin-prefix>:<goal> or <plugin-group-id>:<plugin-artifact-id>[:<plugin-version>]:<goal>. Available lifecycle phases are: validate, initialize, generate-sources, process-sources, generate-resources, process-resources, compile, process-classes, generate-test-sources, process-test-sources, generate-test-resources, process-test-resources, test-compile, process-test-classes, test, prepare-package, package, pre-integration-test, integration-test, post-integration-test, verify, install, deploy, pre-clean, clean, post-clean, pre-site, site, post-site, site-deploy. -> [Help 1]"#, vec!["mvn clean package", "mvn pre-clean package", "mvn post-clean package"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
