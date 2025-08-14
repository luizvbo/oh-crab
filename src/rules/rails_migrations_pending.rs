use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;

const SUGGESTION_REGEX: &str = r"To resolve this issue, run:\s+(.*?)\n";

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(output) = &command.output {
        output.contains("Migrations are pending. To resolve this issue, run:")
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    if let Some(output) = &command.output {
        let migration_script = match Regex::new(SUGGESTION_REGEX).unwrap().captures(output) {
            Some(caps) => caps.get(1).map_or("", |m| m.as_str()),
            None => "",
        };
        vec![system_shell
            .unwrap()
            .and(vec![migration_script, &command.script])]
    } else {
        vec![]
    }
}

pub fn get_rule() -> Rule {
    Rule::new(
        "rails_migrations_pending".to_owned(),
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

    const OUTPUT_ENV_DEVELOPMENT: &str = r#"
Migrations are pending. To resolve this issue, run:

        rails db:migrate RAILS_ENV=development
"#;

    const OUTPUT_ENV_TEST: &str = r#"
Migrations are pending. To resolve this issue, run:

        bin/rails db:migrate RAILS_ENV=test
"#;

    const OUTPUT_MIGRATIONS_PENDING: &str =
        "\nMigrations are pending. To resolve this issue, run:\n\n        {}\n";

    #[rstest]
    #[case("", OUTPUT_ENV_DEVELOPMENT, true)]
    #[case("", OUTPUT_ENV_TEST, true)]
    #[case(
        "Environment data not found in the schema. To resolve this issue, run: \n\n",
        "",
        false
    )]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("bin/rspec", OUTPUT_ENV_DEVELOPMENT, vec!["rails db:migrate RAILS_ENV=development && bin/rspec"])]
    #[case("bin/rspec", OUTPUT_ENV_TEST, vec!["bin/rails db:migrate RAILS_ENV=test && bin/rspec"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, Some(&system_shell)), expected);
    }
}
