use crate::builtin::{try_into_builtin, BuiltInCommandError};
use crate::parser::{Command, Descriptor};
use crate::path::{run_binary, PathError};
use std::collections::HashMap;
use std::io::{stderr, stdout, Write};
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
        let mut descriptors: HashMap<Descriptor, Box<dyn Write>> = HashMap::new();
        descriptors.insert(Descriptor::new(1), Box::new(stdout()));
        descriptors.insert(Descriptor::new(2), Box::new(stderr()));

        if let Ok(builtin) = try_into_builtin(command.program()) {
            // TODO: no stdout hardcoding
            builtin.run(command.arguments(), &mut stdout())?;
        } else {
            // TODO: no descriptors hardcoding
            run_binary(command.program(), command.arguments(), &mut descriptors)?;
        }
    }

    Ok(())
}
