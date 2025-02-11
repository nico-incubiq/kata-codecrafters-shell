use crate::io_redirection::IoRedirections;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn run_binary(
    cmd: &str,
    args: &[String],
    io_redirections: &mut IoRedirections,
) -> Result<(), String> {
    if find_binary_in_path(cmd)?.is_some() {
        let mut command = Command::new(cmd);

        // Pass command args.
        command.args(args);

        // Redirect standard output and error.
        command.stdout(io_redirections.stdout_as_stdio()?);
        command.stderr(io_redirections.stderr_as_stdio()?);

        // Start the program in a thread and wait for it to finish.
        command.status().map(|_| {}).map_err(|e| format!("Command failed: {:?}", e))
    } else {
        Err(format!("{}: command not found", cmd))
    }
}

pub(crate) fn find_binary_in_path(name: &str) -> Result<Option<PathBuf>, String> {
    // Load the PATH env variable.
    let path =
        std::env::var("PATH").map_err(|e| format!("Invalid PATH environment variable: {:?}", e))?;
    let directories = path.split(':');

    // Check whether the file exists in any of the directories.
    let location = directories
        .into_iter()
        .find_map(|dir| Some(Path::new(dir).join(name)).filter(|location| location.exists()));

    Ok(location)
}
