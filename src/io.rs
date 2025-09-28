use crate::parser::{Descriptor, Redirect, RedirectTo};
use std::collections::HashMap;
use std::fs::File;
use std::io::{stderr, stdout, Stderr, Stdout, Write};
use std::process::Stdio;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum IoError {
    #[error("IO error occurred: {0}")]
    StdIo(#[from] std::io::Error),

    #[error("Descriptor {0} is not supported")]
    UnsupportedDescriptor(u8),
}

//TODO: Is an enum really useful here? an opaque struct hiding the Stdout and Stderr would be better.
pub(crate) enum FileDescriptor {
    Stdout(Stdout),
    Stderr(Stderr),
    //TODO: a BufWriter would be efficient for writing, but cannot be converted into Stdio required by process::Command
    File(File),
}

impl FileDescriptor {
    pub(crate) fn stdout() -> Self {
        FileDescriptor::Stdout(stdout())
    }

    pub(crate) fn stderr() -> Self {
        FileDescriptor::Stderr(stderr())
    }

    pub(crate) fn file(filename: &str, append: bool) -> Result<Self, IoError> {
        let file = File::options()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(filename)?;

        Ok(FileDescriptor::File(file))
    }
}

impl From<FileDescriptor> for Stdio {
    fn from(val: FileDescriptor) -> Stdio {
        match val {
            //TODO: might need to wrap in a Lock to allow cloning and having multiple writers?
            FileDescriptor::Stdout(stdout) => stdout.into(),
            FileDescriptor::Stderr(stderr) => stderr.into(),
            FileDescriptor::File(file) => file.into(),
        }
    }
}

impl Write for FileDescriptor {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            FileDescriptor::Stdout(stdout) => stdout.write(buf),
            FileDescriptor::Stderr(stderr) => stderr.write(buf),
            FileDescriptor::File(file) => file.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            FileDescriptor::Stdout(stdout) => stdout.flush(),
            FileDescriptor::Stderr(stderr) => stderr.flush(),
            FileDescriptor::File(file) => file.flush(),
        }
    }
}

pub(crate) fn resolve_redirects(
    redirects: &[Redirect],
) -> Result<HashMap<Descriptor, FileDescriptor>, IoError> {
    //TODO: Before actually opening files, resolve which RedirectTo 1 and 2 go to after going through all redirections, then there's just 2 files to open

    let mut descriptors: HashMap<Descriptor, FileDescriptor> = HashMap::new();

    for redirect in redirects {
        let destination = match redirect.to() {
            RedirectTo::Descriptor(Descriptor(to)) => match to {
                1 => FileDescriptor::stdout(),
                2 => FileDescriptor::stderr(),
                _ => return Err(IoError::UnsupportedDescriptor(to)),
            },
            RedirectTo::File(filename) => FileDescriptor::file(&filename, redirect.append())?,
        };

        descriptors.insert(redirect.from(), destination);
    }

    Ok(descriptors)
}

//TODO: test this:
// -  echo hello '|' world 2> out.txt 1>&2 : writes to out.txt
// -  echo hello '|' world 1>&2 2> out.txt : writes to stdout, because 1>&2 writes to stderr before the redirection is set up
