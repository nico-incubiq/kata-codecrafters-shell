use crate::io::FileDescriptor;
use crate::parser::Descriptor;
use is_executable::IsExecutable;
use std::collections::{HashMap, HashSet};
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
}

pub(crate) fn run_binary(
    cmd: &str,
    args: &[String],
    mut descriptors: HashMap<Descriptor, FileDescriptor>,
) -> Result<(), PathError> {
    let mut command = Command::new(cmd);

    // Pass command args.
    command.args(args);

    // Redirect standard output and error.
    let stdout = descriptors
        .remove(&Descriptor::new(1))
        .unwrap_or(FileDescriptor::stdout());
    let stderr = descriptors
        .remove(&Descriptor::new(2))
        .unwrap_or(FileDescriptor::stderr());

    command.stdout(stdout);
    command.stderr(stderr);

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

/// Finds a file whose name is an exact match in the user PATH.
pub(crate) fn find_file_in_path(name: &str) -> Result<Option<PathBuf>, PathError> {
    // Check whether the file exists in any of the directories.
    let location = get_path_directories()?
        .into_iter()
        .find_map(|dir| Some(dir.join(name)).filter(|location| location.exists()));

    Ok(location)
}

/// Finds executables matching the partial name in the user PATH.
/// This is used for autocompletion, so the start of executable names must match the input.
pub(crate) fn find_partial_executable_matches_in_path(
    partial_name: &str,
) -> Result<HashSet<String>, PathError> {
    let matched_executables: HashSet<_> = get_path_directories()?
        .into_iter()
        // List files in PATH directories, ignoring errors (missing directory, permissions, ...).
        .filter_map(|path| path.read_dir().ok())
        .flatten()
        // Ignore file errors.
        .filter_map(Result::ok)
        // Ignore invalid UTF-8 filenames.
        .filter_map(|file| {
            let file_name = file.file_name().into_string().ok();

            file_name.map(|file_name| (file, file_name))
        })
        // Only keep files for which the start of the name matches the input.
        .filter(|(_, file_name)| file_name.starts_with(partial_name))
        // Only keep executable files.
        .filter(|(file, _)| file.path().is_executable())
        .map(|(_, file_name)| file_name)
        .collect();

    Ok(matched_executables)
}

fn get_path_directories() -> Result<Vec<PathBuf>, PathError> {
    // Load the PATH env variable.
    let path = std::env::var("PATH")?;
    let directories: Vec<_> = path
        .split(':')
        .map(|dir| Path::new(dir).to_path_buf())
        .collect();

    Ok(directories)
}
