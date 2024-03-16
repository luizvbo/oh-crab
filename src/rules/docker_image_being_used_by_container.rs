
use crate::{cli::command::CrabCommand, shell::Shell};
use super::{get_new_command_without_sudo, match_rule_with_is_app, Rule};

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("image is being used by running container")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["docker"], None)
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let container_id = command.output.as_ref().unwrap().trim().split(' ').last().unwrap();
    vec![format!("docker container rm -f {} && {}", container_id, command.script)]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "docker_image_being_used_by_container".to_owned(),
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

    const ERR_RESPONSE: &str = "Error response from daemon: conflict: unable to delete cd809b04b6ff (cannot be forced) - image is being used by running container e5e2591040d1";

    #[rstest]
    #[case("docker image rm -f cd809b04b6ff", ERR_RESPONSE, true)]
    #[case("docker image rm -f cd809b04b6ff", "bash: docker: command not found", false)]
    #[case("git image rm -f cd809b04b6ff", ERR_RESPONSE, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("docker image rm -f cd809b04b6ff", ERR_RESPONSE, vec!["docker container rm -f e5e2591040d1 && docker image rm -f cd809b04b6ff"])]
    fn test_get_new_command(#[case] command: &str, #[case] stdout: &str, #[case] expected: Vec<&str>) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
