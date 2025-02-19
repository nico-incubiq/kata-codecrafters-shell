use crate::builtin::BuiltInCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::fmt::Arguments;
use std::io::{StdoutLock, Write};
use strum::VariantNames;
use thiserror::Error;

const PROMPT: &str = "$ ";

#[derive(Error, Debug)]
pub(crate) enum InputError {
    #[error("Failed to setup raw terminal access: {0:?}")]
    SetupFailed(std::io::Error),

    #[error("Failed to write to the standard output: {0:?}")]
    WriteStdoutFailed(std::io::Error),

    #[error("The user pressed an abortion control sequence")]
    Aborted,
}

pub(crate) fn capture_input() -> Result<String, InputError> {
    // Lock stdout for more repeated writing.
    let mut stdout = std::io::stdout().lock();

    // Prevent the terminal from buffering input, and capture control characters.
    enable_raw_mode().map_err(InputError::SetupFailed)?;

    // Print the prompt.
    write(&mut stdout, format_args!("{}", PROMPT))?;

    let mut input = String::new();

    while let Ok(event) = event::read() {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Tab => {
                    // Find commands that start match the partial input.
                    let matching_commands: Vec<_> = BuiltInCommand::VARIANTS
                        .iter()
                        .filter(|cmd| cmd.starts_with(&input))
                        .collect();

                    // Only autocomplete if exactly one command matches.
                    if 1 == matching_commands.len() {
                        let matching_command = &matching_commands[0];
                        let original_input_len = input.len();

                        // Complete the command in the input buffer and suffix with a space.
                        input.push_str(&matching_command[original_input_len..]);
                        input.push(' ');

                        // Update the terminal accordingly.
                        write(
                            &mut stdout,
                            format_args!("{}", &input[original_input_len..]),
                        )?;
                    }
                }
                KeyCode::Enter => {
                    // Print a carriage return and a new line.
                    write(&mut stdout, format_args!("\r\n"))?;

                    // Stop capture.
                    break;
                }
                KeyCode::Char(c) => {
                    // Handle Ctrl+C to abort current input.
                    if modifiers == KeyModifiers::CONTROL && c == 'c' {
                        // Print a carriage return and a new line.
                        write(&mut stdout, format_args!("\r\n"))?;

                        // Abort.
                        return Err(InputError::Aborted);
                    }

                    // Add the char to the input string buffer and print it to the terminal.
                    input.push(c);
                    write(&mut stdout, format_args!("{}", c))?;
                }
                KeyCode::Backspace => {
                    let original_input_len = input.len();
                    if modifiers == KeyModifiers::CONTROL {
                        // Clear the input completely.
                        //TODO: This branch actually is never hit as some sequences are badly handled
                        //      by crossterm: https://github.com/crossterm-rs/crossterm/issues/685
                        input.clear();
                    } else {
                        // Remove one char from the end of the input.
                        let _ = input.pop();
                    }

                    // Manually clear the removed char(s) from the screen by printing a space in its place.
                    // Print the prompt and the input twice to avoid flashing if clearing it completely the first time.
                    write(
                        &mut stdout,
                        format_args!(
                            "\r{}{}{}\r{}{}",
                            PROMPT,
                            input,
                            " ".repeat(original_input_len - input.len()),
                            PROMPT,
                            input
                        ),
                    )?;
                }
                _ => {
                    // Nothing else is supported for now...
                    eprintln!("Unhandled event: {:?}", event);
                }
            }
        }
    }

    disable_raw_mode().map_err(InputError::SetupFailed)?;

    Ok(input)
}

fn write(stdout: &mut StdoutLock, text: Arguments) -> Result<(), InputError> {
    // Print the text to the terminal buffer and flush it.
    write!(stdout, "{}", text).map_err(InputError::WriteStdoutFailed)?;
    stdout.flush().map_err(InputError::WriteStdoutFailed)?;

    Ok(())
}
