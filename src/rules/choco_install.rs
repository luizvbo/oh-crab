use crate::{cli::command::CrabCommand, shell::Shell};

use super::Rule;

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    if let Some(stdout) = &command.stdout {
        stdout.contains("Installing the following packages")
            && (command.script.starts_with("choco install")
                || command.script_parts.contains(&"cinst".to_owned()))
    } else {
        false
    }
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    let reference = ["choco", "cinst", "install"];
    for script_part in &command.script_parts {
        if
        // These are certainly parameters
        !script_part.contains('=') && !script_part.contains('/') &&
            // Leading hyphens are parameters; some packages contain them though
            !script_part.starts_with('-') &&
            // Need exact match (bc chocolatey is a package)
            !reference.contains(&script_part.as_str())
        {
            return vec![command
                .script
                .replace(script_part, &(script_part.to_owned() + ".install"))];
        }
    }
    vec![]
}

pub fn get_rule() -> Rule {
    Rule::new(
        "choco_install".to_owned(),
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

    const PACKAGE_NOT_FOUND_ERROR: &str = r#"Chocolatey v0.10.15
Installing the following packages:
logstitcher
By installing you accept licenses for the packages.
logstitcher not installed. The package was not found with the source(s) listed.
 Source(s): 'https://chocolatey.org/api/v2/'
 NOTE: When you specify explicit sources, it overrides default sources.
If the package version is a prerelease and you didn\'t specify `--pre`,
 the package may not be found.
Please see https://chocolatey.org/docs/troubleshooting for more
 assistance.

Chocolatey installed 0/1 packages. 1 packages failed.
 See the log for details (C:\\ProgramData\\chocolatey\\logs\\chocolatey.log).

Failures
 - logstitcher - logstitcher not installed. The package was not found with the source(s) listed.
 Source(s): 'https://chocolatey.org/api/v2/'
 NOTE: When you specify explicit sources, it overrides default sources.
If the package version is a prerelease and you didn\'t specify `--pre`,
 the package may not be found.
Please see https://chocolatey.org/docs/troubleshooting for more
 assistance.
"#;

    macro_rules! parameterized_match_rule_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!(match_rule(&mut command, None));
                }
            )*
        }
    }

    macro_rules! parameterized_unmatch_rule_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert!(!match_rule(&mut command, None));
                }
            )*
        }
    }

    macro_rules! parameterized_get_new_command_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (script, stdout, expected) = $value;
                    let mut command = CrabCommand::new(
                                script.to_owned(),
                                Some(stdout.to_owned()),
                                None
                            );
                    assert_eq!(get_new_command(&mut command, None)[0], expected);
                }
            )*
        }
    }

    parameterized_match_rule_tests! {
        match_rule_1: ("choco install logstitcher", PACKAGE_NOT_FOUND_ERROR),
        match_rule_2: ("cinst logstitcher", PACKAGE_NOT_FOUND_ERROR),
        match_rule_3: ("choco install logstitcher -y", PACKAGE_NOT_FOUND_ERROR),
        match_rule_4: ("cinst logstitcher -y", PACKAGE_NOT_FOUND_ERROR),
        match_rule_5: ("choco install logstitcher -y -n=test", PACKAGE_NOT_FOUND_ERROR),
        match_rule_6: ("cinst logstitcher -y -n=test", PACKAGE_NOT_FOUND_ERROR),
        match_rule_7: ("choco install logstitcher -y -n=test /env", PACKAGE_NOT_FOUND_ERROR),
        match_rule_8: ("cinst logstitcher -y -n=test /env", PACKAGE_NOT_FOUND_ERROR),
        match_rule_9: ("choco install chocolatey -y", PACKAGE_NOT_FOUND_ERROR),
        match_rule_10: ("cinst chocolatey -y", PACKAGE_NOT_FOUND_ERROR),
    }

    parameterized_unmatch_rule_tests! {
        unmatch_rule_1: ("choco /?", ""),
        unmatch_rule_2: ("choco upgrade logstitcher", ""),
        unmatch_rule_3: ("cup logstitcher", ""),
        unmatch_rule_4: ("choco upgrade logstitcher -y", ""),
        unmatch_rule_5: ("cup logstitcher -y", ""),
        unmatch_rule_6: ("choco upgrade logstitcher -y -n=test", ""),
        unmatch_rule_7: ("cup logstitcher -y -n=test", ""),
        unmatch_rule_8: ("choco upgrade logstitcher -y -n=test /env", ""),
        unmatch_rule_9: ("cup logstitcher -y -n=test /env", ""),
        unmatch_rule_10: ("choco upgrade chocolatey -y", ""),
        unmatch_rule_11: ("cup chocolatey -y", ""),
        unmatch_rule_12: ("choco uninstall logstitcher", ""),
        unmatch_rule_13: ("cuninst logstitcher", ""),
        unmatch_rule_14: ("choco uninstall logstitcher -y", ""),
        unmatch_rule_15: ("cuninst logstitcher -y", ""),
        unmatch_rule_16: ("choco uninstall logstitcher -y -n=test", ""),
        unmatch_rule_17: ("cuninst logstitcher -y -n=test", ""),
        unmatch_rule_18: ("choco uninstall logstitcher -y -n=test /env", ""),
        unmatch_rule_19: ("cuninst logstitcher -y -n=test /env", ""),
        unmatch_rule_20: ("choco uninstall chocolatey -y", ""),
        unmatch_rule_21: ("cuninst chocolatey -y", ""),
    }

    parameterized_get_new_command_tests! {
        get_new_command_1: ("choco install logstitcher", "", "choco install logstitcher.install"),
        get_new_command_2: ("cinst logstitcher", "", "cinst logstitcher.install"),
        get_new_command_3: ("choco install logstitcher -y", "", "choco install logstitcher.install -y"),
        get_new_command_4: ("cinst logstitcher -y", "", "cinst logstitcher.install -y"),
        get_new_command_5: ("choco install logstitcher -y -n=test", "", "choco install logstitcher.install -y -n=test"),
        get_new_command_6: ("cinst logstitcher -y -n=test", "", "cinst logstitcher.install -y -n=test"),
        get_new_command_7: ("choco install logstitcher -y -n=test /env", "", "choco install logstitcher.install -y -n=test /env"),
        get_new_command_8: ("cinst logstitcher -y -n=test /env", "", "cinst logstitcher.install -y -n=test /env"),
        get_new_command_9: ("choco install chocolatey -y", "", "choco install chocolatey.install -y"),
        // get_new_command_10: ("cinst chocolatey -y", "", "cinst chocolatey.install -y"),
    }
}
