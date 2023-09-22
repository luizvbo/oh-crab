#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod cli;
mod rules;
mod shell;
mod ui;
mod utils;

pub use cli::command;
pub use cli::handler;

const ARGUMENT_PLACEHOLDER: &str = "OHCRAB_ARGUMENT_PLACEHOLDER";
