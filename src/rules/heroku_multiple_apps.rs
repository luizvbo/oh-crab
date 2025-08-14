use super::{utils::match_rule_with_is_app, Rule};
use crate::{cli::command::CrabCommand, shell::Shell};
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

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let re = Regex::new(r"([^ ]*) \([^)]*\)").unwrap();
        let apps = re
            .captures_iter(output)
            .map(|cap| cap[1].to_owned())
            .collect::<Vec<_>>();
        println!("{apps:?}");
        println!("{output:?}");
        apps.iter()
            .map(|app| format!("{} --app {}", command.script, app))
            .collect()
    } else {
        Vec::<String>::new()
    }
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

    const SUGGEST_OUTPUT: &str = r#"
 ▸    Multiple apps in git remotes
 ▸    Usage: --remote heroku-dev
 ▸    or: --app myapp-dev
 ▸    Your local git repository has more than 1 app referenced in git remotes.
 ▸    Because of this, we can't determine which app you want to run this command against.
 ▸    Specify the app you want with --app or --remote.
 ▸    Heroku remotes in repo:
 ▸    myapp (heroku)
 ▸    myapp-dev (heroku-dev)
 ▸
 ▸    https://devcenter.heroku.com/articles/multiple-environments
    "#;

    const NOT_MATCH_OUTPUT: &str = r#"
=== HEROKU_POSTGRESQL_TEAL_URL, DATABASE_URL
Plan:                  Hobby-basic
Status:                Available
Connections:           20/20
PG Version:            9.6.4
Created:               2017-01-01 00:00 UTC
Data Size:             99.9 MB
Tables:                99
Rows:                  12345/10000000 (In compliance)
Fork/Follow:           Unsupported
Rollback:              Unsupported
Continuous Protection: Off
Add-on:                postgresql-round-12345
    "#;

    #[rstest]
    #[case("heroku pg", SUGGEST_OUTPUT, true)]
    #[case("heroku pg", NOT_MATCH_OUTPUT, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("heroku pg", SUGGEST_OUTPUT, vec!["heroku pg --app myapp", "heroku pg --app myapp-dev"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
