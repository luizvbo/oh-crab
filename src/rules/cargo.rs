use which::which;

use crate::command::CrabCommand;

use super::{RuleAttributes, RuleTrait};

pub struct Cargo {
    attributes: RuleAttributes,
}

impl RuleTrait for Cargo {
    // add code here
    fn new(
        &self,
        name: &str,
        enabled_by_default: Option<bool>,
        priority: Option<u16>,
        requires_output: Option<bool>,
    ) {
        self.attributes = RuleAttributes::new(name, enabled_by_default, priority, requires_output)
    }
    fn match_rule(self, command: CrabCommand) -> bool {
        command.script == "cargo"
    }

    fn get_new_command(self, command: CrabCommand) -> String {
        "cargo build".to_owned()
    }
}
