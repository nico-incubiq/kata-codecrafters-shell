use crate::builtin::{try_into_builtin, BuiltInCommandError};
use crate::io::{resolve_redirects};
use crate::parser::Command;
use crate::path::{run_binary, PathError};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum RunnerError {
    #[error(transparent)]
    BuiltInCommand(#[from] BuiltInCommandError),

    #[error(transparent)]
    Path(#[from] PathError),
}

/// Resolves and runs the provided commands, piping stdout of each one into stdin of the next.
pub(crate) fn run_commands(commands: Vec<Command>) -> Result<(), RunnerError> {
    // TODO: pipe commands into each other using https://doc.rust-lang.org/stable/std/io/fn.pipe.html

    for command in commands {
        let descriptors = resolve_redirects(command.redirects());

        if let Ok(builtin) = try_into_builtin(command.program()) {
            // TODO: no stdout hardcoding
            builtin.run(command.arguments(), descriptors)?;
        } else {
            run_binary(command.program(), command.arguments(), descriptors)?;
        }
    }

    Ok(())
}
