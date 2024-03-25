use super::Rule;
use crate::{cli::command::CrabCommand, shell::Shell};

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    command.script.contains("manage.py")
        && command.script.contains("migrate")
        && command.output.as_ref().map_or(false, |output| {
            output.contains("or pass --delete-ghost-migrations")
        })
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    vec![format!("{} --delete-ghost-migrations", command.script)]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "django_south_ghost".to_owned(),
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

    const OUTPUT: &str = r#"Traceback (most recent call last):
  File "/home/nvbn/work/.../bin/python", line 42, in <module>
    exec(compile(__file__f.read(), __file__, "exec"))
  File "/home/nvbn/work/.../app/manage.py", line 34, in <module>
    execute_from_command_line(sys.argv)
  File "/home/nvbn/work/.../lib/django/core/management/__init__.py", line 443, in execute_from_command_line
    utility.execute()
  File "/home/nvbn/work/.../lib/django/core/management/__init__.py", line 382, in execute
    self.fetch_command(subcommand).run_from_argv(self.argv)
  File "/home/nvbn/work/.../lib/django/core/management/base.py", line 196, in run_from_argv
    self.execute(*args, **options.__dict__)
  File "/home/nvbn/work/.../lib/django/core/management/base.py", line 232, in execute
    output = self.handle(*args, **options)
  File "/home/nvbn/work/.../app/lib/south/management/commands/migrate.py", line 108, in handle
    ignore_ghosts = ignore_ghosts,
  File "/home/nvbn/work/.../app/lib/south/migration/__init__.py", line 193, in migrate_app
    applied_all = check_migration_histories(applied_all, delete_ghosts, ignore_ghosts)
  File "/home/nvbn/work/.../app/lib/south/migration/__init__.py", line 88, in check_migration_histories
    raise exceptions.GhostMigrations(ghosts)
south.exceptions.GhostMigrations:

 ! These migrations are in the database but not on disk:
    <app1: 0033_auto__...>
    <app1: 0034_fill_...>
    <app1: 0035_rename_...>
    <app2: 0003_add_...>
    <app2: 0004_denormalize_...>
    <app1: 0033_auto....>
    <app1: 0034_fill...>
 ! I'm not trusting myself; either fix this yourself by fiddling
 ! with the south_migrationhistory table, or pass --delete-ghost-migrations
 ! to South to have it delete ALL of these records (this may not be good).
"#;
    #[rstest]
    #[case("./manage.py migrate", OUTPUT, true)]
    #[case("python manage.py migrate", OUTPUT, true)]
    #[case("./manage.py migrate", "", false)]
    #[case("app migrate", OUTPUT, false)]
    #[case("./manage.py test", OUTPUT, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(match_rule(&mut command, None), is_match);
    }

    #[rstest]
    #[case("./manage.py migrate auth", "", vec!["./manage.py migrate auth --delete-ghost-migrations"])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
