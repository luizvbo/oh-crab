use std::env;

use crate::{ARGUMENT_PLACEHOLDER, ENV_VAR_NAME_ALIAS, ENV_VAR_NAME_HISTORY, ENV_VAR_NAME_SHELL};

pub trait Shell {
    fn app_alias(&self, alias_name: &str) -> String;
}

pub fn get_bash_type(shell_type: &str) -> Box<dyn Shell> {
    let shell_candidate = shell_type.to_lowercase();
    match shell_candidate.as_str() {
        "zsh" => Box::new(Zsh),
        "bash" => Box::new(Bash),
        _ => panic!("The shell '{}' is not supported yet", shell_type),
    }
}

pub struct Zsh;
pub struct Bash;

impl Shell for Zsh {
    fn app_alias(&self, alias_name: &str) -> String {
        format!(
            r#"
            {alias_name} () {{
                export {var_name_shell}="zsh";
                export {var_name_alias}="{alias_name}";
                export {var_name_history}="$(fc -ln -1)";
                OC_CMD=$(
                    ohcrab {argument_placeholder} $@
                ) && eval $OC_CMD;
                unset {var_name_history};
            }}
            "#,
            alias_name = alias_name,
            var_name_shell = ENV_VAR_NAME_SHELL,
            var_name_alias = ENV_VAR_NAME_ALIAS,
            var_name_history = ENV_VAR_NAME_HISTORY,
            argument_placeholder = ARGUMENT_PLACEHOLDER,
        )
    }
}

impl Shell for Bash {
    fn app_alias(&self, alias_name: &str) -> String {
        format!(
            r#"
            function {alias_name} () {{
                export {var_name_shell}="bash";
                export {var_name_alias}="{alias_name}";
                export {var_name_history}="$(fc -ln -1)";
                OC_CMD=$(
                    ohcrab {argument_placeholder} "$@"
                ) && eval "$OC_CMD";
                unset {var_name_history};
            }}
            "#,
            alias_name = alias_name,
            var_name_history = ENV_VAR_NAME_HISTORY,
            var_name_shell = ENV_VAR_NAME_SHELL,
            var_name_alias = ENV_VAR_NAME_ALIAS,
            argument_placeholder = ARGUMENT_PLACEHOLDER,
        )
    }
}
