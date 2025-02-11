use crate::io_redirection::{IoRedirectionError, IoRedirections};
use std::env::VarError;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum PathError {
    #[error("{0}: command not found")]
    CommandNotFound(String),

    #[error("{0}: execution failed: {1:?}")]
    CommandError(String, std::io::Error),

    #[error("Failed to read environment variable: {0}")]
    GetEnvFailed(#[from] VarError),

    #[error("Failed setting up standard I/O redirection: {0}")]
    IoRedirectionFailed(#[from] IoRedirectionError),
}

pub(crate) fn run_binary(
    cmd: &str,
    args: &[String],
    io_redirections: &mut IoRedirections,
) -> Result<(), PathError> {
    let mut command = Command::new(cmd);

    // Pass command args.
    command.args(args);

    // Redirect standard output and error.
    command.stdout(io_redirections.stdout_as_stdio()?);
    command.stderr(io_redirections.stderr_as_stdio()?);

    // Start the program in a thread and wait for it to finish, ignoring the exit status.
    let _ = command.status().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PathError::CommandNotFound(cmd.to_owned())
        } else {
            PathError::CommandError(cmd.to_owned(), e)
        }
    })?;

    Ok(())
}

pub(crate) fn find_in_path(name: &str) -> Result<Option<PathBuf>, PathError> {
    // Load the PATH env variable.
    let path = std::env::var("PATH")?;
    let directories = path.split(':');

    // Check whether the file exists in any of the directories.
    let location = directories
        .into_iter()
        .find_map(|dir| Some(Path::new(dir).join(name)).filter(|location| location.exists()));

    Ok(location)
}
