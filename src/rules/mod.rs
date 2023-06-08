pub mod rules;

pub trait RuleTrait {
    fn match_rule(&self, command: CrabCommand) -> bool;
    fn get_new_command(&self, command: CrabCommand) -> String;
    fn side_effect(&self, command: CrabCommand, fixed_command: String) {}
}

pub struct Rule {
    name: String,
    enabled_by_default: bool,
    priority: u16,
    requires_output: bool,
}

impl Rule {
    pub fn new(
        name: String,
        enabled_by_default: Option<bool>,
        priority: Option<u16>,
        requires_output: Option<bool>,
    ) -> Self {
        Self {
            name,
            enabled_by_default: enabled_by_default.unwrap_or(true),
            priority: priority.unwrap_or(0),
            requires_output: requires_output.unwrap_or(true),
        }
    }
}

pub fn get_rules() -> Vec<Rule> {
    vec![Rule::new("apt_get", None)]
}
