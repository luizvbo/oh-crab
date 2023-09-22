use std::env;

use crate::ARGUMENT_PLACEHOLDER;

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
            {name} () {{
                OC_PREVIOUS=$(fc -ln -1 | tail -n 1);
                OC_CMD=$(
                    OC_ALIAS={name}
                    OC_SHELL_ALIASES=$(alias)
                    ohcrab $OC_PREVIOUS {argument_placeholder} --bash zsh $*
                ) && eval $OC_CMD;
                {alter_history}
            }}
            "#,
            name = alias_name,
            argument_placeholder = ARGUMENT_PLACEHOLDER,
            alter_history = "test -n \"$TF_CMD\" && print -s $TF_CMD"
        )
    }
}

impl Shell for Bash {
    fn app_alias(&self, alias_name: &str) -> String {
        format!(
            r#"
            function {name} () {{
                OC_PREVIOUS=$(fc -ln -1);
                export OC_ALIAS={name};
                export OC_SHELL_ALIASES=$(alias);
                export PYTHONIOENCODING=utf-8;
                OC_CMD=$(
                    ohcrab $OC_PREVIOUS {argument_placeholder} --bash bash $@
                ) && eval $OC_CMD;
                {alter_history}
            }}
            "#,
            name = alias_name,
            argument_placeholder = ARGUMENT_PLACEHOLDER,
            alter_history = "test -n \"$TF_CMD\" && print -s $TF_CMD"
        )
    }
}
