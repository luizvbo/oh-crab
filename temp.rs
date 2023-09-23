use console::{style, Key, Term};
use std::io::{self, Write};

enum UpdateOptions {
    Increment,
    Decrement,
    JustPrint,
}

fn update_item(items: &Vec<String>, mut index: usize, increment: UpdateOptions) -> usize {
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
    print!(
        "\rThis is the item {}",
        items.get(index).unwrap_or(&"".to_owned())
    );
    io::stdout().flush().unwrap(); // Flush to ensure it's immediately displayed
    index
}

fn iterative_menu(items: &Vec<String>) -> Option<&String> {
    if items.is_empty() {
        None
    } else {
        let mut index = 0;
        index = update_item(items, index, UpdateOptions::JustPrint);
        // index = (index + 1) % numbers.len();
        let stdout = Term::buffered_stdout();
        'game_loop: loop {
            if let Ok(character) = stdout.read_key() {
                match character {
                    Key::ArrowUp => index = update_item(items, index, UpdateOptions::Increment),
                    Key::ArrowDown => index = update_item(items, index, UpdateOptions::Decrement),
                    Key::Char(c) => match c {
                        'k' => index = update_item(items, index, UpdateOptions::Increment),
                        'j' => index = update_item(items, index, UpdateOptions::Decrement),
                        _ => (),
                    },
                    Key::Enter => break 'game_loop,
                    _ => (),
                }
            }
        }
        Some(&items[index])
    }
}

fn main() {
    let s = vec![
        "A".to_owned(),
        "B".to_owned(),
        "C".to_owned(),
        "D".to_owned(),
    ];
    iterative_menu(&s);
}
