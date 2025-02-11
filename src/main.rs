mod builtin;
mod io_redirection;
mod path;
mod quoting;

use crate::builtin::{BuiltInCommand, BuiltInCommandError};
use crate::io_redirection::{handle_io_redirections, IoRedirectionError};
use crate::path::{run_binary, PathError};
use crate::quoting::{split_quoted_string, QuotingError};
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug)]
enum ShellError {
    #[error("Failed to print the prompt: {0:?}")]
    PrintPromptFailed(std::io::Error),

    #[error("Failed to read the input: {0:?}")]
    ReadInputFailed(std::io::Error),

    #[error("{0}: {1}")]
    BuiltIn(String, BuiltInCommandError),

    #[error(transparent)]
    IoRedirection(#[from] IoRedirectionError),

    #[error(transparent)]
    Path(#[from] PathError),

    #[error(transparent)]
    Quoting(#[from] QuotingError),
}

fn main() {
    loop {
        if let Err(error) = repl() {
            // Print any error that couldn't be printed to the potential stderr redirection.
            eprintln!("{}", error);
        }
    }
}

fn repl() -> Result<(), ShellError> {
    // Print the prompt and grab the input.
    let input = input_prompt()?;

    // Split the command and arguments.
    let (command, mut args) = match parse_input(&input)? {
        Some(input) => input,
        None => return Ok(()),
    };

    // Get the standard output / error descriptors to execute the command.
    let mut io_redirections = handle_io_redirections(&mut args)?;

    // Interpret the command name and run it.
    if let Err(e) = match BuiltInCommand::try_from(command.as_ref()) {
        Ok(built_in) => built_in
            .run(&args, &mut io_redirections)
            .map_err(|e| ShellError::BuiltIn(built_in.name(), e)),
        _ => run_binary(&command, &args, &mut io_redirections).map_err(ShellError::Path),
    } {
        // Write errors to the standard error.
        io_redirections.ewriteln(format_args!("{}", e))?;
    }

    Ok(())
}

fn input_prompt() -> Result<String, ShellError> {
    // Print the prompt.
    print!("$ ");
    std::io::stdout()
        .flush()
        .map_err(ShellError::PrintPromptFailed)?;

    // Wait for user input.
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(ShellError::ReadInputFailed)?;

    Ok(input.trim().to_owned())
}

/// Parse the input string into a command and its arguments.
fn parse_input(input: &str) -> Result<Option<(String, Vec<String>)>, ShellError> {
    let mut values = split_quoted_string(input)?;

    if values.is_empty() {
        Ok(None)
    } else {
        Ok(Some((values.remove(0), values)))
    }
}
