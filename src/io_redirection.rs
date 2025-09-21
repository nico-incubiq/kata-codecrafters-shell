use std::fs::{File, OpenOptions};
use std::io::{stderr, stdout, Write};
use std::process::Stdio;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum IoRedirectionError {
    #[error("Failed to write to standard I/O: {0:?}")]
    StandardIoWriteFailed(#[source] std::io::Error),

    #[error("Failed to write to file '{0}': {0:?}")]
    FileWriteFailed(String, #[source] std::io::Error),

    #[error("Failed to open file '{0}': {1:?}")]
    FileOpenFailed(String, #[source] std::io::Error),

    #[error("Failed to clone handle for file '{0}': {1:?}")]
    FileHandleCloneFailed(String, #[source] std::io::Error),

    #[error("Missing file name for standard output redirect")]
    MissingArgument,
}

/// Looks for stdout and/or stderr redirections in the args.
/// Also removes the redirections from the args list.
///
/// # Note
/// Redirections are expected to be surrounded with spaces.
pub(crate) fn handle_io_redirections(
    args: &mut Vec<String>,
) -> Result<IoRedirections, IoRedirectionError> {
    // Look for standard output redirect.
    let stdout = if let Some((file_name, append)) = extract_io_redirection(args, 1)? {
        let handle = open_redirect_file(&file_name, append)?;
        Descriptor::File(file_name, handle)
    } else {
        Descriptor::StandardOutput
    };

    // Look for standard error redirect.
    let stderr = if let Some((file_name, append)) = extract_io_redirection(args, 2)? {
        let handle = open_redirect_file(&file_name, append)?;
        Descriptor::File(file_name, handle)
    } else {
        Descriptor::StandardError
    };

    Ok(IoRedirections { stdout, stderr })
}

fn open_redirect_file(file_name: &str, append: bool) -> Result<File, IoRedirectionError> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(!append)
        .append(append)
        .open(file_name)
        .map_err(|e| IoRedirectionError::FileOpenFailed(file_name.to_owned(), e))
}

/// Looks for an IO redirection in the args.
/// Also removes the redirection from the args list.
fn extract_io_redirection(
    args: &mut Vec<String>,
    descriptor: u8,
) -> Result<Option<(String, bool)>, IoRedirectionError> {
    for index in 0..args.len() {
        if args[index] == format!("{descriptor}>")
            || args[index] == format!("{descriptor}>>")
            || 1 == descriptor && (args[index] == ">" || args[index] == ">>")
        {
            if index == args.len() - 1 {
                return Err(IoRedirectionError::MissingArgument);
            }

            // Remove the filename and the redirect operator from the list of args.
            let file_name = args.remove(index + 1);
            let redirect_operator = args.remove(index);

            return Ok(Some((file_name, redirect_operator.ends_with(">>"))));
        }
    }

    Ok(None)
}

pub(crate) struct IoRedirections {
    stdout: Descriptor,
    stderr: Descriptor,
}

impl IoRedirections {
    /// Writes a new line to the standard output.
    pub(crate) fn writeln(&mut self, args: std::fmt::Arguments) -> Result<(), IoRedirectionError> {
        self.stdout.writeln(args)
    }

    /// Writes a new line to the standard error.
    pub(crate) fn ewriteln(&mut self, args: std::fmt::Arguments) -> Result<(), IoRedirectionError> {
        self.stderr.writeln(args)
    }

    pub(crate) fn stdout_as_stdio(&mut self) -> Result<Stdio, IoRedirectionError> {
        (&mut self.stdout).try_into()
    }

    pub(crate) fn stderr_as_stdio(&mut self) -> Result<Stdio, IoRedirectionError> {
        (&mut self.stderr).try_into()
    }
}

enum Descriptor {
    /// File name and the descriptor to write to it.
    File(String, File),
    StandardOutput,
    StandardError,
}

impl Descriptor {
    fn writeln(&mut self, args: std::fmt::Arguments) -> Result<(), IoRedirectionError> {
        match self {
            Descriptor::File(name, file) => writeln!(file, "{args}")
                .map_err(|e| IoRedirectionError::FileWriteFailed(name.to_owned(), e)),
            Descriptor::StandardOutput | Descriptor::StandardError => {
                writeln!(stdout(), "{args}").map_err(IoRedirectionError::StandardIoWriteFailed)
            }
        }
    }
}

impl TryFrom<&mut Descriptor> for Stdio {
    type Error = IoRedirectionError;

    fn try_from(value: &mut Descriptor) -> Result<Self, Self::Error> {
        let stdio = match value {
            Descriptor::File(name, file) => {
                let file = file
                    .try_clone()
                    .map_err(|e| IoRedirectionError::FileHandleCloneFailed(name.to_owned(), e))?;

                file.into()
            }
            Descriptor::StandardOutput => stdout().into(),
            Descriptor::StandardError => stderr().into(),
        };

        Ok(stdio)
    }
}

#[cfg(test)]
mod tests {
    use crate::io_redirection::{extract_io_redirection, IoRedirectionError};

    #[test]
    fn it_extracts_redirect() {
        // Redirects of stdout with and without the descriptor id.
        assert_eq!(
            Some(("test.txt".to_owned(), false)),
            extract_io_redirection(
                &mut ["hello", "1>", "test.txt"].map(str::to_owned).to_vec(),
                1
            )
            .unwrap()
        );
        assert_eq!(
            Some(("test.txt".to_owned(), false)),
            extract_io_redirection(
                &mut ["hello", ">", "test.txt"].map(str::to_owned).to_vec(),
                1
            )
            .unwrap()
        );

        // Redirects of stderr.
        assert_eq!(
            Some(("test.txt".to_owned(), false)),
            extract_io_redirection(
                &mut ["hello", "2>", "test.txt"].map(str::to_owned).to_vec(),
                2
            )
            .unwrap()
        );

        // Handles redirects at any position in the arguments.
        assert_eq!(
            Some(("test.txt".to_owned(), false)),
            extract_io_redirection(
                &mut ["hello", ">", "test.txt", "world"]
                    .map(str::to_owned)
                    .to_vec(),
                1
            )
            .unwrap()
        );
    }

    #[test]
    fn it_errors_if_filename_is_missing() {
        assert!(matches!(
            extract_io_redirection(&mut ["hello", ">"].map(str::to_owned).to_vec(), 1),
            Err(IoRedirectionError::MissingArgument),
        ));
    }

    #[test]
    fn it_removes_args_pertaining_to_the_redirection() {
        let mut args = ["hello", "1>", "test.txt"].map(str::to_owned).to_vec();
        let _ = extract_io_redirection(&mut args, 1);
        assert_eq!(["hello".to_owned()].to_vec(), args);
    }

    #[test]
    fn it_extracts_appending_redirect() {
        // Redirects of stdout with and without the descriptor id.
        assert_eq!(
            Some(("test.txt".to_owned(), true)),
            extract_io_redirection(
                &mut ["hello", "1>>", "test.txt"].map(str::to_owned).to_vec(),
                1
            )
            .unwrap()
        );
        assert_eq!(
            Some(("test.txt".to_owned(), true)),
            extract_io_redirection(
                &mut ["hello", ">>", "test.txt"].map(str::to_owned).to_vec(),
                1
            )
            .unwrap()
        );

        // Redirects of stderr.
        assert_eq!(
            Some(("test.txt".to_owned(), true)),
            extract_io_redirection(
                &mut ["hello", "2>>", "test.txt"].map(str::to_owned).to_vec(),
                2
            )
            .unwrap()
        );
    }
}
