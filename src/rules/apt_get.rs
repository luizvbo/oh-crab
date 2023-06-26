use crate::{
    command::CrabCommand,
    rules::{RuleAttributes, RuleTrait},
};

pub struct AptGet {
    attributes: RuleAttributes,
}

fn _get_executable(command: CrabCommand) -> String {
    if command.script_parts[0] == "sudo" {
        command.script_parts[1]
    } else {
        command.script_parts[0]
    }
}

impl RuleTrait for AptGet {
    fn new(
        name: &str,
        enabled_by_default: Option<bool>,
        priority: Option<u16>,
        requires_output: Option<bool>,
    ) -> AptGet {
        AptGet {
            attributes: RuleAttributes::new(name, enabled_by_default, priority, requires_output),
        }
    }

    fn match_rule(&self, command: CrabCommand) -> bool {
        if command.stderr.contains("not found") | command.stderr.contains("not installed") {
            let executable = _get_executable(command);
            // not which(executable) and get_package(executable)
        } else {
            false
        }
    }
}
