use crate::command::{CorrectedCommand, CrabCommand};
use console::{style, Key, Term};
use std::io::{self, Write};

/// Enum representing different options for updating an item.
enum UpdateOptions {
    Increment,
    Decrement,
    JustPrint,
}

/// Displays the confirmation text for a given corrected command.
///
/// # Arguments
///
/// * `command` - A reference to a `CorrectedCommand`.
pub fn confirm_text(command: &CorrectedCommand) {
    let prefix = "\u{200B}".repeat(10);
    print!(
        "\r{}{}{}{}{}{}{}{}{}{}{}{}",
        prefix,
        style(command.script.to_owned()).bold(),
        if command.side_effect.is_some() {
            " (+side_effect)"
        } else {
            ""
        },
        " [",
        style("enter").green(),
        " | ",
        style("↑/k").blue(),
        " | ",
        style("↓/j").blue(),
        " | ",
        style("CTRL+c").red(),
        "]"
    );
}

/// Updates the index based on the chosen update option.
///
/// # Arguments
///
/// * `items` - A reference to a vector of `CorrectedCommand`.
/// * `index` - The current index to be updated.
/// * `increment` - The update option, either Increment, Decrement, or JustPrint.
///
/// # Returns
///
/// The updated index after applying the specified update option.
fn update_item(items: &Vec<CorrectedCommand>, mut index: usize, increment: UpdateOptions) -> usize {
    match increment {
        UpdateOptions::Increment => index = (index + 1) % items.len(),
        UpdateOptions::Decrement => {
            index = if index == 0 {
                items.len() - 1
            } else {
                index - 1
            }
        }
        UpdateOptions::JustPrint => (),
    }
    io::stdout().flush().unwrap(); // Flush to ensure it's immediately displayed
    if let Some(correct_command) = items.get(index) {
        confirm_text(correct_command);
    }
    io::stdout().flush().unwrap(); // Flush again to display the new content
    index
}

/// Implements an iterative menu for selecting from a list of corrected commands.
///
/// # Arguments
///
/// * `corrected_commands` - A reference to a vector of `CorrectedCommand`.
///
/// # Returns
///
/// An optional reference to the selected `CorrectedCommand`.
pub fn iterative_menu<'a>(
    corrected_commands: &'a Vec<CorrectedCommand>,
) -> Option<&'a CorrectedCommand> {
    if corrected_commands.is_empty() {
        None
    } else {
        let mut index = 0;
        index = update_item(&corrected_commands, index, UpdateOptions::JustPrint);
        // index = (index + 1) % numbers.len();
        let stdout = Term::buffered_stdout();
        'game_loop: loop {
            if let Ok(character) = stdout.read_key() {
                match character {
                    Key::ArrowUp => {
                        index = update_item(&corrected_commands, index, UpdateOptions::Increment)
                    }
                    Key::ArrowDown => {
                        index = update_item(&corrected_commands, index, UpdateOptions::Decrement)
                    }
                    Key::Char(c) => match c {
                        'k' => {
                            index =
                                update_item(&corrected_commands, index, UpdateOptions::Increment)
                        }
                        'j' => {
                            index =
                                update_item(&corrected_commands, index, UpdateOptions::Decrement)
                        }
                        _ => (),
                    },
                    Key::Enter => break 'game_loop,
                    _ => (),
                }
            }
        }
        corrected_commands.get(index)
    }
}
