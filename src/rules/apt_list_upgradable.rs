use crate::{cli::command::CrabCommand, shell::Shell};

use super::{get_new_command_without_sudo, match_without_sudo, Rule};

fn _match_rule(command: &CrabCommand) -> bool {
    if let Some(stdout) = &command.output {
        stdout.contains("apt list --upgradable")
    } else {
        false
    }
}

pub fn match_rule(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> bool {
    match_without_sudo(_match_rule, command)
}

fn _get_new_command(command: &CrabCommand) -> Vec<String> {
    vec!["apt list --upgradable".to_owned()]
}

pub fn get_new_command(command: &mut CrabCommand, system_shell: Option<&dyn Shell>) -> Vec<String> {
    get_new_command_without_sudo(_get_new_command, command)
}

pub fn get_rule() -> Rule {
    Rule::new(
        "apt_list_upgradable".to_owned(),
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

    const FULL_ENGLISH_OUTPUT: &str = r#"
Hit:1 http://us.archive.ubuntu.com/ubuntu zesty InRelease
Hit:2 http://us.archive.ubuntu.com/ubuntu zesty-updates InRelease
Get:3 http://us.archive.ubuntu.com/ubuntu zesty-backports InRelease [89.2 kB]
Hit:4 http://security.ubuntu.com/ubuntu zesty-security InRelease
Hit:5 http://ppa.launchpad.net/ubuntu-mozilla-daily/ppa/ubuntu zesty InRelease
Hit:6 https://download.docker.com/linux/ubuntu zesty InRelease
Hit:7 https://cli-assets.heroku.com/branches/stable/apt ./ InRelease
Fetched 89.2 kB in 0s (122 kB/s)
Reading package lists... Done
Building dependency tree
Reading state information... Done
8 packages can be upgraded. Run 'apt list --upgradable' to see them.
"#;

    const NO_MATCH_OUTPUT: &str = r#"
Hit:1 http://us.archive.ubuntu.com/ubuntu zesty InRelease
Get:2 http://us.archive.ubuntu.com/ubuntu zesty-updates InRelease [89.2 kB]
Get:3 http://us.archive.ubuntu.com/ubuntu zesty-backports InRelease [89.2 kB]
Get:4 http://security.ubuntu.com/ubuntu zesty-security InRelease [89.2 kB]
Hit:5 https://cli-assets.heroku.com/branches/stable/apt ./ InRelease
Hit:6 http://ppa.launchpad.net/ubuntu-mozilla-daily/ppa/ubuntu zesty InRelease
Hit:7 https://download.docker.com/linux/ubuntu zesty InRelease
Get:8 http://us.archive.ubuntu.com/ubuntu zesty-updates/main i386 Packages [232 kB]
Get:9 http://us.archive.ubuntu.com/ubuntu zesty-updates/main amd64 Packages [235 kB]
Get:10 http://us.archive.ubuntu.com/ubuntu zesty-updates/main amd64 DEP-11 Metadata [55.2 kB]
Get:11 http://us.archive.ubuntu.com/ubuntu zesty-updates/main DEP-11 64x64 Icons [32.3 kB]
Get:12 http://us.archive.ubuntu.com/ubuntu zesty-updates/universe amd64 Packages [156 kB]
Get:13 http://us.archive.ubuntu.com/ubuntu zesty-updates/universe i386 Packages [156 kB]
Get:14 http://us.archive.ubuntu.com/ubuntu zesty-updates/universe amd64 DEP-11 Metadata [175 kB]
Get:15 http://us.archive.ubuntu.com/ubuntu zesty-updates/universe DEP-11 64x64 Icons [253 kB]
Get:16 http://us.archive.ubuntu.com/ubuntu zesty-updates/multiverse amd64 DEP-11 Metadata [5,840 B]
Get:17 http://us.archive.ubuntu.com/ubuntu zesty-backports/universe amd64 DEP-11 Metadata [4,588 B]
Get:18 http://security.ubuntu.com/ubuntu zesty-security/main amd64 DEP-11 Metadata [12.7 kB]
Get:19 http://security.ubuntu.com/ubuntu zesty-security/main DEP-11 64x64 Icons [17.6 kB]
Get:20 http://security.ubuntu.com/ubuntu zesty-security/universe amd64 DEP-11 Metadata [21.6 kB]
Get:21 http://security.ubuntu.com/ubuntu zesty-security/universe DEP-11 64x64 Icons [47.7 kB]
Get:22 http://security.ubuntu.com/ubuntu zesty-security/multiverse amd64 DEP-11 Metadata [208 B]
Fetched 1,673 kB in 0s (1,716 kB/s)
Reading package lists... Done
Building dependency tree
Reading state information... Done
All packages are up to date.
"#;

    const GERMAN_OUTPUT: &str = "Führen Sie »apt list --upgradable« aus, um sie anzuzeigen.";

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
        match_rule_1: ("sudo apt update", FULL_ENGLISH_OUTPUT),
        match_rule_2: ("sudo apt update", GERMAN_OUTPUT),
    }

    parameterized_unmatch_rule_tests! {
    unmatch_rule_1: ("sudo apt update", NO_MATCH_OUTPUT),
    unmatch_rule_2: ("apt-cache search foo", ""),
    unmatch_rule_3: ("aptitude search foo", ""),
    unmatch_rule_4: ("apt search foo", ""),
    unmatch_rule_5: ("apt-get install foo", ""),
    unmatch_rule_6: ("apt-get source foo", ""),
    unmatch_rule_7: ("apt-get clean", ""),
    unmatch_rule_8: ("apt-get remove", ""),
    unmatch_rule_9: ("apt-get update", ""),
    }

    parameterized_get_new_command_tests! {
        get_new_command_1: ("sudo apt update", FULL_ENGLISH_OUTPUT, "sudo apt list --upgradable"),
        get_new_command_2: ("sudo apt update", GERMAN_OUTPUT, "sudo apt list --upgradable"),
        get_new_command_3: ("apt update", FULL_ENGLISH_OUTPUT, "apt list --upgradable"),
        get_new_command_4: ("apt update", GERMAN_OUTPUT, "apt list --upgradable"),
    }
}
