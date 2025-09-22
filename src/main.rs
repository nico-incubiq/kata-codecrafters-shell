mod autocomplete;
mod builtin;
mod input;
mod io_redirection;
mod parser;
mod path;
mod runner;

use crate::autocomplete::CompositeAutocomplete;
use crate::builtin::BuiltInCommandError;
use crate::input::{capture_input, InputError};
use crate::parser::{parse_input, ParsingError};
use crate::runner::{run_commands, RunnerError};
use std::process::exit;
use thiserror::Error;

#[derive(Error, Debug)]
enum ShellError {
    #[error(transparent)]
    Autocomplete(#[from] InputError),

    #[error(transparent)]
    Parsing(#[from] ParsingError),

    #[error(transparent)]
    Runner(#[from] RunnerError),
}

fn main() {
    loop {
        if let Err(error) = repl() {
            match error {
                ShellError::Runner(RunnerError::BuiltInCommand(BuiltInCommandError::Exit(
                    code,
                ))) => exit(code),
                // Print any error that couldn't be printed to the potential stderr redirection.
                error => eprintln!("{error}"),
            }
        }
    }
}

fn repl() -> Result<(), ShellError> {
    // Initialise autocompletion.
    let autocomplete = CompositeAutocomplete::new();

    // Capture the user input.
    let input = match capture_input(&autocomplete) {
        // Start a new repl iteration on abortion.
        Err(InputError::Aborted) => return Ok(()),
        res => res?,
    };

    // Parse the commands.
    let commands = parse_input(&input)?;
    if commands.is_empty() {
        return Ok(());
    }

    run_commands(commands)?;

    Ok(())
}
