mod autocomplete;
mod builtin;
mod parsing;
mod input;
mod io_redirection;
mod path;
mod quoting;

use crate::autocomplete::CompositeAutocomplete;
use crate::builtin::{try_into_builtin, BuiltInCommandError};
use crate::input::{capture_input, InputError};
use crate::io_redirection::{handle_io_redirections, IoRedirectionError};
use crate::path::{run_binary, PathError};
use thiserror::Error;
// use crate::parsing::{parse_commands_from_input, ParsingError};

#[derive(Error, Debug)]
enum ShellError {
    #[error(transparent)]
    Autocomplete(#[from] InputError),

    #[error("{0}: {1}")]
    BuiltIn(String, BuiltInCommandError),

    // #[error(transparent)]
    // Parsing(#[from] ParsingError),

    #[error(transparent)]
    IoRedirection(#[from] IoRedirectionError),

    #[error(transparent)]
    Path(#[from] PathError),
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
    // Initialise autocompletion.
    let autocomplete = CompositeAutocomplete::new();

    // Capture the user input.
    let input = match capture_input(autocomplete) {
        // Start a new repl iteration on abortion.
        Err(InputError::Aborted) => return Ok(()),
        res => res?,
    };

    // Split the commands and arguments.
    // let commands = parse_commands_from_input(&input)?;
    // if commands.is_empty() {
    //     return Ok(())
    // }
    //
    // //TODO: Start every command in parallel, and connect the output of one to the input of the next, so streaming works.
    //
    // // Get the standard output / error descriptors to execute the commands.
    // let mut io_redirections = handle_io_redirections(&mut args)?;
    //
    // // Interpret the command name and run it.
    // if let Err(e) = match try_into_builtin(command.as_ref()) {
    //     Ok(built_in) => built_in
    //         .run(&args, &mut io_redirections)
    //         .map_err(|e| ShellError::BuiltIn(built_in.to_string(), e)),
    //     _ => run_binary(&command, &args, &mut io_redirections).map_err(ShellError::Path),
    // } {
    //     // Write errors to the standard error.
    //     io_redirections.ewriteln(format_args!("{}", e))?;
    // }

    Ok(())
}
