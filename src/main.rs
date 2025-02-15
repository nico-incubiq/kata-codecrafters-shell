mod builtin;
mod input;
mod io_redirection;
mod path;
mod quoting;

use crate::builtin::{BuiltInCommand, BuiltInCommandError};
use crate::input::{capture_input, InputError};
use crate::io_redirection::{handle_io_redirections, IoRedirectionError};
use crate::path::{run_binary, PathError};
use crate::quoting::{split_quoted_string, QuotingError};
use thiserror::Error;

#[derive(Error, Debug)]
enum ShellError {
    #[error(transparent)]
    Autocomplete(#[from] InputError),

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
    // Capture the user input.
    let input = match capture_input() {
        // Start new repl iteration on abortion.
        Err(InputError::Aborted) => return Ok(()),
        res => res?,
    };

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

/// Parse the input string into a command and its arguments.
fn parse_input(input: &str) -> Result<Option<(String, Vec<String>)>, ShellError> {
    let mut values = split_quoted_string(input)?;

    if values.is_empty() {
        Ok(None)
    } else {
        Ok(Some((values.remove(0), values)))
    }
}
