use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("No goals have been specified for this build")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["mvn"], None)
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![
        format!("{} clean package", command.script),
        format!("{} clean install", command.script),
    ]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "mvn_no_command".to_owned(),
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

    const ERROR_NO_GOALS: &str = "[ERROR] No goals have been specified for this build. You must specify a valid lifecycle phase or a goal in the format <plugin-prefix>:<goal> or <plugin-group-id>:<plugin-artifact-id>[:<plugin-version>]:<goal>. Available lifecycle phases are: validate, initialize, generate-sources, process-sources, generate-resources, process-resources, compile, process-classes, generate-test-sources, process-test-sources, generate-test-resources, process-test-resources, test-compile, process-test-classes, test, prepare-package, package, pre-integration-test, integration-test, post-integration-test, verify, install, deploy, pre-clean, clean, post-clean, pre-site, site, post-site, site-deploy. -> [Help 1]";

    #[rstest]
    #[case("mvn", ERROR_NO_GOALS, true)]
    #[case("mvn clean", "\n[INFO] Scanning for projects...[INFO]\n[INFO] ------------------------------------------------------------------------\n[INFO] Building test 0.2\n[INFO] ------------------------------------------------------------------------\n[INFO]\n[INFO] --- maven-clean-plugin:2.5:clean (default-clean) @ test ---\n[INFO] Deleting /home/mlk/code/test/target\n[INFO] ------------------------------------------------------------------------\n[INFO] BUILD SUCCESS\n[INFO] ------------------------------------------------------------------------\n[INFO] Total time: 0.477s\n[INFO] Finished at: Wed Aug 26 13:05:47 BST 2015\n[INFO] Final Memory: 6M/240M\n[INFO] ------------------------------------------------------------------------", false)]
    #[case("mvn --help", "", false)]
    #[case("mvn -v", "", false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("mvn", ERROR_NO_GOALS, vec!["mvn clean package", "mvn clean install"])]
    #[case("mvn -N", ERROR_NO_GOALS, vec!["mvn -N clean package", "mvn -N clean install"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
