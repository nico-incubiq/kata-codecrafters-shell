use std::fs::{File, OpenOptions};
use std::io::{stderr, stdout, Write};
use std::process::Stdio;

pub(crate) fn handle_io_redirections(args: &mut Vec<String>) -> Result<IoRedirections, String> {
    // Look for standard output redirect.
    let stdout = if let Some(file_name) = extract_io_redirection(args, 1)? {
        Descriptor::File(open_redirect_file(&file_name, true)?)
    } else {
        Descriptor::StandardOutput
    };

    // Look for standard error redirect.
    let stderr = if let Some(file_name) = extract_io_redirection(args, 2)? {
        Descriptor::File(open_redirect_file(&file_name, true)?)
    } else {
        Descriptor::StandardError
    };

    Ok(IoRedirections { stdout, stderr })
}

fn open_redirect_file(file_name: &str, truncate: bool) -> Result<File, String> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(truncate)
        .open(&file_name)
        .map_err(|e| format!("Failed to redirect to '{}': {:?}", file_name, e))
}

/// Looks for an IO redirection and removes it from the args list.
fn extract_io_redirection(
    args: &mut Vec<String>,
    descriptor: u8,
) -> Result<Option<String>, String> {
    if let Some(index) = args
        .iter()
        .position(|arg| arg == &format!("{}>", descriptor) || descriptor == 1 && arg == ">")
    {
        if index == args.len() - 1 {
            return Err("Missing file name for standard output redirect".to_owned());
        }

        let file_name = args.remove(index + 1);
        args.remove(index);

        Ok(Some(file_name))
    } else {
        Ok(None)
    }
}

pub(crate) struct IoRedirections {
    stdout: Descriptor,
    stderr: Descriptor,
}

impl IoRedirections {
    /// Writes a new line to the standard output.
    pub(crate) fn writeln(&mut self, args: std::fmt::Arguments) -> Result<(), String> {
        self.stdout.writeln(args)
    }

    /// Writes a new line to the standard error.
    pub(crate) fn ewriteln(&mut self, args: std::fmt::Arguments) -> Result<(), String> {
        self.stderr.writeln(args)
    }

    pub(crate) fn stdout_as_stdio(&mut self) -> Result<Stdio, String> {
        (&mut self.stdout).try_into()
    }

    pub(crate) fn stderr_as_stdio(&mut self) -> Result<Stdio, String> {
        (&mut self.stderr).try_into()
    }
}

enum Descriptor {
    File(File),
    StandardOutput,
    StandardError,
}

impl Descriptor {
    fn writeln(&mut self, args: std::fmt::Arguments) -> Result<(), String> {
        match self {
            Descriptor::File(file) => {
                writeln!(file, "{}", args).map_err(|e| format!("Failed to write to file: {:?}", e))
            }
            Descriptor::StandardOutput | Descriptor::StandardError => {
                writeln!(stdout(), "{}", args)
                    .map_err(|e| format!("Failed to write to standard output: {:?}", e))
            }
        }
    }
}

impl TryFrom<&mut Descriptor> for Stdio {
    type Error = String;

    fn try_from(value: &mut Descriptor) -> Result<Self, Self::Error> {
        let stdio = match value {
            Descriptor::File(file) => {
                let file = file
                    .try_clone()
                    .map_err(|e| format!("Failed to clone file descriptor: {:?}", e))?;

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
    use crate::io_redirection::extract_io_redirection;

    #[test]
    fn it_extracts_redirect() {
        // Redirects of stdout with and without the descriptor id.
        assert_eq!(
            Ok(Some("test.txt".to_owned())),
            extract_io_redirection(
                &mut ["hello", "1>", "test.txt"].map(str::to_owned).to_vec(),
                1
            )
        );
        assert_eq!(
            Ok(Some("test.txt".to_owned())),
            extract_io_redirection(
                &mut ["hello", ">", "test.txt"].map(str::to_owned).to_vec(),
                1
            )
        );

        // Redirects of stderr.
        assert_eq!(
            Ok(Some("test.txt".to_owned())),
            extract_io_redirection(
                &mut ["hello", "2>", "test.txt"].map(str::to_owned).to_vec(),
                2
            )
        );
    }

    #[test]
    fn it_errors_if_filename_is_missing() {
        assert_eq!(
            Err("Missing file name for standard output redirect".to_owned()),
            extract_io_redirection(&mut ["hello", ">"].map(str::to_owned).to_vec(), 1)
        );
    }

    #[test]
    fn it_removes_args_pertaining_to_the_redirection() {
        let mut args = ["hello", "1>", "test.txt"].map(str::to_owned).to_vec();
        let _ = extract_io_redirection(&mut args, 1);
        assert_eq!(["hello".to_owned()].to_vec(), args);
    }
}
