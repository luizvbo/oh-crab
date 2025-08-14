use crate::cli::command::CorrectedCommand;
use console::{style, Key, Term};
use std::io::{self, Write};

/// Displays the confirmation text for a given corrected command.
///
/// # Arguments
///
/// * `command` - A reference to a `CorrectedCommand`.
pub fn confirm_text(command: &CorrectedCommand) {
    let prefix = "\r\x1B[K";
    eprint!(
        "\r{}{}{} [{}|{}|{}|{}]",
        prefix,
        style(command.script.to_owned()).for_stderr().bold(),
        if command.side_effect.is_some() {
            " (+side_effect)"
        } else {
            ""
        },
        style("enter").for_stderr().green(),
        style("↑/k").for_stderr().blue(),
        style("↓/j").for_stderr().blue(),
        style("CTRL+c").for_stderr().red()
    );
}

/// Implements an interactive menu for selecting from a list of corrected commands.
///
/// # Arguments
///
/// * `corrected_commands` - A reference to a vector of `CorrectedCommand`.
///
/// # Returns
///
/// An optional reference to the selected `CorrectedCommand`.
pub fn interactive_menu(corrected_commands: &[CorrectedCommand]) -> Option<&CorrectedCommand> {
    if corrected_commands.is_empty() {
        return None;
    }

    let mut index = 0;
    let term = Term::stderr();
    let num_items = corrected_commands.len();

    let draw_menu = |index: usize| {
        if let Some(command) = corrected_commands.get(index) {
            confirm_text(command);
            io::stderr().flush().unwrap();
        }
    };

    draw_menu(index);

    loop {
        if let Ok(key) = term.read_key() {
            match key {
                Key::ArrowUp | Key::Char('k') => {
                    index = (index + num_items - 1) % num_items;
                }
                Key::ArrowDown | Key::Char('j') => {
                    index = (index + 1) % num_items;
                }
                Key::Enter => {
                    return corrected_commands.get(index);
                }
                Key::Char(c) => {
                    // Clear the line before exiting
                    let prefix = "\r\x1B[K";
                    eprint!("{}", prefix);
                    return None;
                }
                _ => {}
            }
            draw_menu(index);
        }
    }
}
