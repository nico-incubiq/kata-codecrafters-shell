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
            // Disengage multi-autocomplete if any other key than tab is pressed.
            if code != KeyCode::Tab {
                multi_autocomplete_on = false;
            }

            match code {
                KeyCode::Tab => {
                    // Attempt to autocomplete the input.
                    let completions = autocomplete.completions(&input)?;

                    // Only autocomplete if exactly 1 completion was found.
                    if let Some(completion) =
                        completions.iter().next().filter(|_| completions.len() == 1)
                    {
                        let original_input_len = input.len();

                        // Complete the command in the input buffer and suffix with a space.
                        input.push_str(&completion[original_input_len..]);
                        input.push(' ');

                        // Update the terminal accordingly.
                        write(
                            &mut stdout,
                            format_args!("{}", &input[original_input_len..]),
                        )?;
                    } else if completions.len() > 1 && multi_autocomplete_on {
                        let mut completions: Vec<_> = completions.iter().cloned().collect();
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
