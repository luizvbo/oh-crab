use super::{get_new_command_without_sudo, utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("docker")
            && output.contains("access denied")
            && output.contains("may require 'docker login'")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["docker"], None)
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    vec!["docker login && ".to_owned() + &command.script]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "docker_login".to_owned(),
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

    const ERR_RESPONSE1: &str = "r#
    Sending build context to Docker daemon  118.8kB
Step 1/6 : FROM foo/bar:fdb7c6d
pull access denied for foo/bar, repository does not exist or may require 'docker login'
#";

    const ERR_RESPONSE2: &str = "r#
    The push refers to repository [artifactory:9090/foo/bar]
push access denied for foo/bar, repository does not exist or may require 'docker login'
#";
    const ERR_RESPONSE3: &str = "r#
    docker push artifactory:9090/foo/bar:fdb7c6d
The push refers to repository [artifactory:9090/foo/bar]
9c29c7ad209d: Preparing
71f3ad53dfe0: Preparing
f58ee068224c: Preparing
aeddc924d0f7: Preparing
c2040e5d6363: Preparing
4d42df4f350f: Preparing
35723dab26f9: Preparing
71f3ad53dfe0: Pushed
cb95fa0faeb1: Layer already exists
#";

    #[rstest]
    #[case(
        "docker build -t artifactory:9090/foo/bar:fdb7c6d .",
        ERR_RESPONSE1,
        true
    )]
    #[case("docker push artifactory:9090/foo/bar:fdb7c6d", ERR_RESPONSE2, true)]
    #[case("docker push artifactory:9090/foo/bar:fdb7c6d", ERR_RESPONSE3, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("docker build -t artifactory:9090/foo/bar:fdb7c6d .", "", vec!["docker login && docker build -t artifactory:9090/foo/bar:fdb7c6d ."])]
    #[case("docker push artifactory:9090/foo/bar:fdb7c6d", "", vec!["docker login && docker push artifactory:9090/foo/bar:fdb7c6d"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
