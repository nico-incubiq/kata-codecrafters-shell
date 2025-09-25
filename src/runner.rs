use crate::builtin::{try_into_builtin, BuiltInCommandError};
use crate::io::FileDescriptor;
use crate::parser::{Command, Descriptor};
use crate::path::{run_binary, PathError};
use std::collections::HashMap;
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
        // TODO: no descriptors hardcoding
        let mut descriptors: HashMap<Descriptor, FileDescriptor> = HashMap::new();
        descriptors.insert(Descriptor::new(1), FileDescriptor::stdout());
        descriptors.insert(Descriptor::new(2), FileDescriptor::stderr());

        if let Ok(builtin) = try_into_builtin(command.program()) {
            // TODO: no stdout hardcoding
            builtin.run(command.arguments(), &mut FileDescriptor::stdout())?;
        } else {
            run_binary(command.program(), command.arguments(), descriptors)?;
        }
    }

    Ok(())
}

//TODO: test this:
// -  echo hello '|' world 2> out.txt 1>&2 : writes to out.txt
// -  echo hello '|' world 1>&2 2> out.txt : writes to stdout, because 1>&2 writes to stderr before the redirection is set up
