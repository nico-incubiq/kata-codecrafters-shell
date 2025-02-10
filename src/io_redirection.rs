use std::fs::File;
use std::io::{stderr, stdout, Write};
use std::process::Stdio;

pub(crate) fn get_io_redirections() -> Result<IoRedirections, String> {
    // (IoRedirection::StandardOutput, IoRedirection::StandardOutput)
    Ok(IoRedirections {
        stdout: Descriptor::File(File::create("./stdout.txt").unwrap()),
        stderr: Descriptor::File(File::create("./stderr.txt").unwrap()),
    })
}

pub(crate) struct IoRedirections {
    stdout: Descriptor,
    stderr: Descriptor,
}

impl IoRedirections {
    pub(crate) fn writeln(&mut self, args: std::fmt::Arguments) -> Result<(), String> {
        self.stdout.writeln(args)
    }

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
                    .map_err(|e| format!("Failed to clone file handle: {:?}", e))?;

                file.into()
            },
            Descriptor::StandardOutput => stdout().into(),
            Descriptor::StandardError => stderr().into(),
        };

        Ok(stdio)
    }
}
