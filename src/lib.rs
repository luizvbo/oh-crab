#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod cli;
pub mod rules;
pub mod shell;
mod ui;
mod utils;

const ARGUMENT_PLACEHOLDER: &str = "OHCRAB_ARGUMENT_PLACEHOLDER";
const ENV_VAR_NAME_HISTORY: &str = "OHCRAB_COMMAND_HISTORY";
const ENV_VAR_NAME_ALIAS: &str = "OHCRAB_ALIAS";
const ENV_VAR_NAME_SHELL: &str = "OHCRAB_SHELL";
