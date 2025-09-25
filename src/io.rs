use std::collections::HashMap;
use std::fs::File;
use std::io::{stderr, stdout, Stderr, Stdout, Write};
use std::process::Stdio;
use thiserror::Error;
use crate::parser::{Descriptor, Redirect};

#[derive(Error, Debug)]
pub(crate) enum IoError {
    #[error("IO error occurred: {0}")]
    StdIo(#[from] std::io::Error),
}

//TODO: Is an enum really useful here? an opaque struct hiding the Stdout and Stderr would be better.
pub(crate) enum FileDescriptor {
    Stdout(Stdout),
    Stderr(Stderr),
    //TODO: a BufWriter would be efficient for writing, but cannot be converted into Stdio required by process::Command
    File(String, File),
}

impl FileDescriptor {
    pub(crate) fn stdout() -> Self {
        FileDescriptor::Stdout(stdout())
    }

    pub(crate) fn stderr() -> Self {
        FileDescriptor::Stderr(stderr())
    }

    pub(crate) fn file(filename: &str) -> Result<Self, IoError> {
        let file = File::open(filename)?;

        Ok(FileDescriptor::File(filename.to_owned(), file))
    }
}

impl From<FileDescriptor> for Stdio {
    fn from(val: FileDescriptor) -> Stdio {
        match val {
            //TODO: might need to wrap in a Lock to allow cloning and having multiple writers?
            FileDescriptor::Stdout(stdout) => stdout.into(),
            FileDescriptor::Stderr(stderr) => stderr.into(),
            FileDescriptor::File(filename, file) => file.into(),
        }
    }
}

impl Write for FileDescriptor {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            FileDescriptor::Stdout(stdout) => stdout.write(buf),
            FileDescriptor::Stderr(stderr) => stderr.write(buf),
            FileDescriptor::File(_, file) => file.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            FileDescriptor::Stdout(stdout) => stdout.flush(),
            FileDescriptor::Stderr(stderr) => stderr.flush(),
            FileDescriptor::File(_, file) => file.flush(),
        }
    }
}

pub(crate) fn resolve_redirects(redirects: &[Redirect]) -> HashMap<Descriptor, FileDescriptor> {
    let mut descriptors: HashMap<Descriptor, FileDescriptor> = HashMap::new();
    descriptors.insert(Descriptor::stdout(), FileDescriptor::stdout());
    descriptors.insert(Descriptor::stderr(), FileDescriptor::stderr());

    descriptors
}

//TODO: test this:
// -  echo hello '|' world 2> out.txt 1>&2 : writes to out.txt
// -  echo hello '|' world 1>&2 2> out.txt : writes to stdout, because 1>&2 writes to stderr before the redirection is set up
