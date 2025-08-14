use crate::{cli::command::CrabCommand, shell::Shell};
use regex::Regex;
use shlex::split as shlex_split;
use shlex::Quoter;

use super::is_app;

/// Provides git support for a given function.
///
/// # Arguments
///
/// * `func` - A function that takes a `CrabCommand` instance and returns a boolean.
/// * `command` - A mutable `CrabCommand` instance.
///
/// # Returns
///
/// * `bool` - Returns the result of the function `func`.
pub fn match_rule_with_git_support<F>(func: F, command: &mut CrabCommand) -> bool
where
    F: Fn(&CrabCommand) -> bool,
{
    // supports GitHub's `hub` command
    // which is recommended to be used with `alias git=hub`
    // but at this point, shell aliases have already been resolved
    if !is_app(command, vec!["git", "hub"], None) {
        return false;
    }

    // perform git aliases expansion
    if let Some(stdout) = &command.output {
        if stdout.contains("trace: alias expansion:") {
            let re = Regex::new(r"trace: alias expansion: ([^ ]*) => ([^\n]*)").unwrap();
            if let Some(search) = re.captures(stdout) {
                let alias = search.get(1).map_or("", |m| m.as_str());

                // by default git quotes everything, for example:
                //     'commit' '--amend'
                // which is surprising and does not allow to easily test for
                // eg. 'git commit'
                let expansion = search
                    .get(2)
                    .map_or("", |m| m.as_str())
                    .split_whitespace()
                    .map(|part| format!("\"{part}\"")) // shell.quote(part)
                    .collect::<Vec<_>>()
                    .join(" ");
                let new_script = command.script.replace(&format!(r"\b{alias}\b"), &expansion);

                command.script = new_script;
            }
        }
    }

    func(command)
}

pub fn get_new_command_with_git_support<F>(
    func: F,
    command: &mut CrabCommand,
    system_shell: Option<&dyn Shell>,
) -> Vec<String>
where
    F: Fn(&CrabCommand, Option<&dyn Shell>) -> Vec<String>,
{
    // supports GitHub's `hub` command
    // which is recommended to be used with `alias git=hub`
    // but at this point, shell aliases have already been resolved
    if !is_app(command, vec!["git", "hub"], None) {
        return Vec::<String>::new();
    }
    let mut new_command = command;

    // perform git aliases expansion
    if let Some(stdout) = &new_command.output {
        if stdout.contains("trace: alias expansion:") {
            let re = Regex::new(r"trace: alias expansion: ([^ ]*) => ([^\n]*)").unwrap();
            if let Some(search) = re.captures(stdout) {
                let shlex_quoter = Quoter::new();
                let alias = search.get(1).map_or("", |m| m.as_str());

                // by default git quotes everything, for example:
                //     'commit' '--amend'
                // which is surprising and does not allow to easily test for
                // eg. 'git commit'
                let expansion = search.get(2).map_or("", |m| m.as_str());
                let expansion = shlex_split(expansion)
                    .unwrap()
                    .iter()
                    .map(|s| shlex_quoter.quote(s).unwrap())
                    .collect::<Vec<_>>()
                    .join(" ");

                let re = Regex::new(&format!(r"\b{alias}\b")).unwrap();
                let new_script = re.replace(&new_command.script, &expansion);

                *new_command = new_command.update(Some(new_script.to_string()), None, None);
            }
        }
    }

    func(new_command, system_shell)
}

#[cfg(test)]
mod tests {
    use super::{get_new_command_with_git_support, is_app, match_rule_with_git_support};
    use crate::cli::command::CrabCommand;
    use crate::shell::Shell;
    use rstest::rstest;

    #[rstest]
    #[case("/usr/bin/git diff", vec!["git", "hub"], None, true)]
    #[case("/bin/hdfs dfs -rm foo", vec!["hdfs"], None, true)]
    #[case("git diff", vec!["git", "hub"], None, true)]
    #[case("hub diff", vec!["git", "hub"], None, true)]
    #[case("hg diff", vec!["git", "hub"], None, false)]
    #[case("/path/to/app run", vec!["app"], Some(1), true)]
    #[case("/path/to/app run", vec!["app"], Some(0), true)]
    #[case("/path/to/app", vec!["app"], None, true)]
    #[case("/path/to/app", vec!["app"], Some(0), true)]
    #[case("/path/to/app", vec!["app"], Some(1), false)]
    fn test_is_app(
        #[case] script: &str,
        #[case] app_names: Vec<&str>,
        #[case] at_least: Option<usize>,
        #[case] is_app_bool: bool,
    ) {
        let mut command = CrabCommand::new(script.to_owned(), None, None);
        assert_eq!(is_app(&command, app_names, at_least), is_app_bool);
    }

    #[rstest]
    #[case(
        "git co",
        "19:22:36.299340 git.c:282   trace: alias expansion: co => 'checkout'",
        "git checkout"
    )]
    #[case(
        "git com file",
        "19:23:25.470911 git.c:282   trace: alias expansion: com => 'commit' '--verbose'",
        "git commit --verbose file"
    )]
    #[case(
        "git com -m \"Initial commit\"",
        "19:22:36.299340 git.c:282   trace: alias expansion: com => \"commit\"",
        "git commit -m \"Initial commit\""
    )]
    #[case(
        "git br -d some_branch",
        "19:22:36.299340 git.c:282   trace: alias expansion: br => 'branch'",
        "git branch -d some_branch"
    )]
    fn test_get_command_with_git_support(
        #[case] script: &str,
        #[case] output: &str,
        #[case] expected: &str,
    ) {
        let mut command = CrabCommand::new(script.to_owned(), Some(output.to_owned()), None);
        let func =
            |command: &CrabCommand, shell: Option<&dyn Shell>| vec![command.script.to_owned()];
        assert_eq!(
            get_new_command_with_git_support(func, &mut command, None),
            vec![expected]
        );
    }

    #[rstest]
    #[case("git pull", Some("".to_string()), true)]
    #[case("hub pull", Some("".to_owned()), true)]
    #[case("git push --set-upstream origin foo", Some("".to_owned()), true)]
    #[case("hub push --set-upstream origin foo", Some("".to_owned()), true)]
    #[case("ls", Some("".to_owned()), false)]
    #[case("cat git", Some("".to_owned()), false)]
    #[case("cat hub", Some("".to_owned()), false)]
    #[case("git pull", None, true)]
    #[case("hub pull", None, true)]
    #[case("git push --set-upstream origin foo", None, true)]
    #[case("hub push --set-upstream origin foo", None, true)]
    #[case("ls", None, false)]
    #[case("cat git", None, false)]
    #[case("cat hub", None, false)]
    fn test_match_rule_with_git_support(
        #[case] script: &str,
        #[case] output: Option<String>,
        #[case] is_git_command: bool,
    ) {
        let mut command = CrabCommand::new(script.to_owned(), output, None);
        let func = |command: &CrabCommand| true;
        assert_eq!(
            match_rule_with_git_support(func, &mut command),
            is_git_command
        );
    }
}
