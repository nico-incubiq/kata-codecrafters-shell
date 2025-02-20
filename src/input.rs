use crate::autocomplete::{Autocomplete, AutocompleteError};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::fmt::Arguments;
use std::io::{StdoutLock, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum InputError {
    #[error("Failed to setup raw terminal access: {0:?}")]
    SetupFailed(std::io::Error),

    #[error("Failed to write to the standard output: {0:?}")]
    WriteStdoutFailed(std::io::Error),

    #[error("Autocomplete failed: {0}")]
    Autocomplete(#[from] AutocompleteError),

    #[error("The user pressed an abortion control sequence")]
    Aborted,
}

/// Takes control of the terminal to capture the input.
/// Note: this puts the terminal in raw mode and handles every keystroke.
pub(crate) fn capture_input(autocomplete: impl Autocomplete) -> Result<String, InputError> {
    // Lock stdout for more repeated writing.
    let mut stdout = std::io::stdout().lock();

    // Prevent the terminal from buffering input, and capture control characters.
    enable_raw_mode().map_err(InputError::SetupFailed)?;

    // Print the prompt.
    write(&mut stdout, build_prompt())?;

    // Handles double-presses of TAB to display multiple autocompletes.
    let mut multi_autocomplete_on = false;

    let mut input = String::new();

    while let Ok(event) = event::read() {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            // Disengage multi-autocomplete if any other key than TAB is pressed.
            if code != KeyCode::Tab {
                multi_autocomplete_on = false;
            }

            match code {
                KeyCode::Tab => {
                    let original_input_len = input.len();

                    // Look for completions for the input.
                    let mut completions: Vec<_> =
                        autocomplete.completions(&input)?.into_iter().collect();

                    if !completions.is_empty() {
                        let longest_prefix = longest_prefix(&completions);

                        // Partially autocomplete to the longest common completions prefix.
                        input.push_str(&longest_prefix[original_input_len..]);

                        // Update the terminal accordingly.
                        write(
                            &mut stdout,
                            format_args!("{}", &input[original_input_len..]),
                        )?;
                    }

                    if completions.len() == 1 {
                        // If exactly 1 completion was found, append a space after the command.
                        input.push(' ');

                        // Update the terminal accordingly.
                        write(&mut stdout, format_args!(" "))?;
                    } else if completions.len() > 1 && multi_autocomplete_on {
                        // Print all completions if multiple were found and TAB was pressed twice.
                        completions.sort();

                        // Print a new line below the current one, print all the completions, then
                        // print the prompt and current input again.
                        write(
                            &mut stdout,
                            format_args!(
                                "\r\n{}\r\n{}{}",
                                completions.join("  "),
                                build_prompt(),
                                input
                            ),
                        )?;
                    } else {
                        // No completion found or multiple completions but pressed TAB only once.
                        ring_terminal_bell(&mut stdout)?;
                    }

                    // Toggle multi-autocompletion, or disable it if len <= 1.
                    multi_autocomplete_on = completions.len() > 1 && !multi_autocomplete_on;
                }
                KeyCode::Enter => {
                    // Print a carriage return and a new line.
                    write(&mut stdout, format_args!("\r\n"))?;

                    // Stop capture.
                    break;
                }
                KeyCode::Char(character) => {
                    match (modifiers, character) {
                        (KeyModifiers::CONTROL, 'c') => {
                            // Print a carriage return and a new line.
                            write(&mut stdout, format_args!("\r\n"))?;

                            // Handle Ctrl+C to abort current repl input.
                            return Err(InputError::Aborted);
                        }
                        (KeyModifiers::CONTROL, 'j') => {
                            // Print a carriage return and a new line.
                            write(&mut stdout, format_args!("\r\n"))?;

                            // Handle Ctrl+J similarly to `Enter`.
                            break;
                        }
                        (KeyModifiers::NONE, _) | (KeyModifiers::SHIFT, _) => {
                            // Add the char to the input string buffer and print it to the terminal.
                            input.push(character);
                            write(&mut stdout, format_args!("{}", character))?;
                        }
                        _ => {
                            // Ignore unknown sequences.
                            continue;
                        }
                    }
                }
                KeyCode::Backspace => {
                    let original_input_len = input.len();
                    if modifiers == KeyModifiers::CONTROL {
                        // Clear the input completely.
                        // TODO: This branch is never hit as some sequences are badly handled by
                        //       crossterm: https://github.com/crossterm-rs/crossterm/issues/685
                        input.clear();
                    } else {
                        // Remove one char from the end of the input.
                        let _ = input.pop();
                    }

                    let prompt = build_prompt();
                    let removed_chars = original_input_len - input.len();

                    // Manually clear the removed char(s) from the screen by printing spaces.
                    // Print the prompt and the input twice to avoid flashing.
                    write(
                        &mut stdout,
                        format_args!(
                            "\r{}{}{}\r{}{}",
                            prompt,
                            input,
                            " ".repeat(removed_chars),
                            prompt,
                            input
                        ),
                    )?;
                }
                _ => {
                    // Nothing else is supported for now...
                }
            }
        }
    }

    disable_raw_mode().map_err(InputError::SetupFailed)?;

    Ok(input)
}

fn longest_prefix(completions: &[String]) -> String {
    let first_completion = completions
        .first()
        .map(|c| c.to_owned())
        .unwrap_or_default();

    // Look for the first char of the first completion which is not common to all completions.
    for (index, char) in first_completion.chars().enumerate() {
        for completion in completions {
            if !completion.chars().nth(index).is_some_and(|c| c == char) {
                return first_completion[0..index].to_owned();
            }
        }
    }

    first_completion
}

/// Builds the prompt.
fn build_prompt() -> Arguments<'static> {
    format_args!("$ ")
}

/// Rings the terminal bell.
fn ring_terminal_bell(stdout: &mut StdoutLock) -> Result<(), InputError> {
    // Print the `\a` character to ring a bell if no completion exists.
    write(stdout, format_args!("{}", 0x07 as char))
}

/// Outputs text to the terminal.
fn write(stdout: &mut StdoutLock, text: Arguments) -> Result<(), InputError> {
    // Print the text to the terminal buffer and flush it.
    write!(stdout, "{}", text).map_err(InputError::WriteStdoutFailed)?;
    stdout.flush().map_err(InputError::WriteStdoutFailed)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::input::longest_prefix;

    #[test]
    fn it_finds_longest_prefix() {
        // No completion in the list.
        assert_eq!("", longest_prefix(&[]));

        // Just one completion in the list.
        assert_eq!("e", longest_prefix(&["e"].map(str::to_owned)));

        // Multiple completions sharing a few common chars.
        assert_eq!("e", longest_prefix(&["echo", "exit"].map(str::to_owned)));
        assert_eq!(
            "echo",
            longest_prefix(&["echo", "echo_two"].map(str::to_owned))
        );
        assert_eq!("ec", longest_prefix(&["echo", "ec"].map(str::to_owned)));

        // Multiple completions with no common chars.
        assert_eq!("", longest_prefix(&["echo", "write"].map(str::to_owned)));
        assert_eq!("", longest_prefix(&["echo", "w"].map(str::to_owned)));
    }
}
