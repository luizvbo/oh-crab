use super::{
    utils::{
        match_rule_with_is_app,
        npm::{mockable_get_scripts, run_npm_command},
    },
    Rule,
};
use crate::{cli::command::CrabCommand, shell::Shell};

fn mockable_match_rule<F>(command: &CrabCommand, fn_run_npm_command: F) -> bool
where
    F: Fn() -> Vec<u8>,
{
    if let Some(output) = &command.output {
        println!("{:?}", output.contains("Usage: npm <command>"));
        println!(
            "{:?}",
            !command
                .script_parts
                .iter()
                .any(|part| part.starts_with("ru"))
        );
        println!("{:?}", mockable_get_scripts(&fn_run_npm_command));
        println!("{:?}", &command.script_parts[1]);
        println!(
            "{:?}",
            mockable_get_scripts(&fn_run_npm_command).contains(&command.script_parts[1])
        );

        output.contains("Usage: npm <command>")
            && !command
                .script_parts
                .iter()
                .any(|part| part.starts_with("ru"))
            && mockable_get_scripts(fn_run_npm_command).contains(&command.script_parts[1])
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_rule_with_is_app(
        |command| mockable_match_rule(command, run_npm_command),
        command,
        vec!["npm"],
        None,
    )
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let mut parts = command.script_parts.clone();
    parts.insert(1, "run-script".to_owned());
    vec![parts.join(" ")]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "npm_run_script".to_owned(),
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
    use super::{get_new_command, mockable_match_rule};
    use crate::cli::command::CrabCommand;
    use crate::rules::utils::match_rule_with_is_app;
    use crate::shell::Bash;
    use rstest::rstest;

    const OUTPUT: &str = r#"Usage: npm <command>

where <command> is one of:
    access, add-user, adduser, apihelp, author, bin, bugs, c,
    cache, completion, config, ddp, dedupe, deprecate, dist-tag,
    dist-tags, docs, edit, explore, faq, find, find-dupes, get,
    help, help-search, home, i, info, init, install, issues, la,
    link, list, ll, ln, login, logout, ls, outdated, owner,
    pack, ping, prefix, prune, publish, r, rb, rebuild, remove,
    repo, restart, rm, root, run-script, s, se, search, set,
    show, shrinkwrap, star, stars, start, stop, t, tag, team,
    test, tst, un, uninstall, unlink, unpublish, unstar, up,
    update, upgrade, v, version, view, whoami

npm <cmd> -h     quick help on <cmd>
npm -l           display full usage info
npm faq          commonly asked questions
npm help <term>  search for help on <term>
npm help npm     involved overview

Specify configs in the ini-formatted file:
    /home/nvbn/.npmrc
or on the command line via: npm <command> --key value
Config info can be viewed via: npm help config"#;

    const RUN_SCRIPT_STDOUT: &[u8] = b"
Lifecycle scripts included in code-view-web:
  test
    jest

available via `npm run-script`:
  build
    cp node_modules/ace-builds/src-min/ -a resources/ace/ && webpack --progress --colors -p --config ./webpack.production.config.js
  develop
    cp node_modules/ace-builds/src/ -a resources/ace/ && webpack-dev-server --progress --colors
  watch-test
    jest --verbose --watch";

    fn mocked_run_npm_command() -> Vec<u8> {
        RUN_SCRIPT_STDOUT.to_vec()
    }

    #[rstest]
    #[case("npm watch-test", OUTPUT, true)]
    #[case("npm develop", OUTPUT, true)]
    #[case("npm test", "TEST FAIL", false)]
    #[case("npm watch-test", "TEST FAIL", false)]
    #[case("npm test", OUTPUT, false)]
    #[case("vim watch-test", OUTPUT, false)]
    fn test_match(#[case] command: &str, #[case] stdout: &str, #[case] is_match: bool) {
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(
            match_rule_with_is_app(
                |command| mockable_match_rule(command, mocked_run_npm_command),
                &command,
                vec!["npm"],
                None,
            ),
            is_match
        );
    }

    #[rstest]
    #[case("npm watch-test", OUTPUT, vec!["npm run-script watch-test"])]
    #[case("npm -i develop", OUTPUT, vec!["npm run-script -i develop"])]
    #[case("npm -i watch-script --path ..", OUTPUT, vec!["npm run-script -i watch-script --path .."])]
    fn test_get_new_command(
        #[case] command: &str,
        #[case] stdout: &str,
        #[case] expected: Vec<&str>,
    ) {
        let system_shell = Bash {};
        let mut command = CrabCommand::new(command.to_owned(), Some(stdout.to_owned()), None);
        assert_eq!(get_new_command(&mut command, None), expected);
    }
}
