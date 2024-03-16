use crate::{cli::command::CrabCommand, shell::Shell};
use super::{get_new_command_without_sudo, match_rule_with_is_app, Rule};
use regex::Regex;

fn auxiliary_match_rule(command: &CrabCommand) -> bool {
    if let Some(output) = &command.output {
        output.contains("https://devcenter.heroku.com/articles/multiple-environments")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(auxiliary_match_rule, command, vec!["heroku"], None)
}

pub fn auxiliary_get_new_command(command: &CrabCommand) -> Vec<String> {
    let re = Regex::new(r"([^ ]*) \\([^)]*\\)").unwrap();
    let apps = re.captures_iter(&command.output.as_ref().unwrap()).map(|cap| cap[1].to_owned()).collect::<Vec<_>>();
    apps.iter().map(|app| format!("{} --app {}", command.script, app)).collect()
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(auxiliary_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "heroku_multiple_apps".to_owned(),
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

    const SUGGEST_OUTPUT: &str = "\n ▸    Multiple apps in git remotes\n ▸    Usage: --remote heroku-dev\n ▸    or: --app myapp-dev\n ▸    Your local git repository has more than 1 app referenced in git remotes.\n ▸    Because of this, we can't determine which app you want to run this command against.\n ▸    Specify the app you want with --app or --remote.\n ▸    Heroku remotes in repo:\n ▸    myapp (heroku)\n ▸    myapp-dev (heroku-dev)\n ▸\n ▸    https://devcenter.heroku.com/articles/multiple-environments\n";
    const NOT_MATCH_OUTPUT: &str = "\n=== HEROKU_POSTGRESQL_TEAL_URL, DATABASE_URL\nPlan:                  Hobby-basic\nStatus:                Available\nConnections:           20/20\nPG Version:            9.6.4\nCreated:               2017-01-01 00:00 UTC\nData Size:             99.9 MB\nTables:                99\nRows:                  12345/10000000 (In compliance)\nFork/Follow:           Unsupported\nRollback:              Unsupported\nContinuous Protection: Off\nAdd-on:                postgresql-round-12345\n";

    #[rstest]
    #[case("heroku pg", SUGGEST_OUTPUT, true)]
    #[case("heroku pg", NOT_MATCH_OUTPUT, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("heroku pg", SUGGEST_OUTPUT, vec!["heroku pg --app myapp", "heroku pg --app myapp-dev"])]
    fn test_get_new_command(#[case] command: &str, #[case] stdout: &str, #[case] expected: Vec<&str>) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
