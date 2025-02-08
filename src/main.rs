use std::fmt::{Display, Formatter};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    loop {
        // Print the prompt and grab the command.
        let input = prompt();

        // Split the command and arguments.
        let (command, args) = input
            .trim()
            .split_once(' ')
            .map(|(cmd, args)| (cmd, Some(args)))
            .unwrap_or((input.trim(), None));

        match (BuiltInCommand::try_from(command), args) {
            (Ok(BuiltInCommand::Exit), Some("0")) => break,
            (Ok(BuiltInCommand::Echo), _) => println!("{}", args.unwrap_or_default()),
            (Ok(BuiltInCommand::Pwd), None) => println!("{}", std::env::current_dir().unwrap().display()),
            (Ok(BuiltInCommand::Type), Some(args)) => match BuiltInCommand::try_from(args) {
                Ok(arg_command) => {
                    println!("{} is a shell builtin", arg_command)
                }
                _ => type_of_binary(args),
            },
            _ => run_binary(command, args),
        }
    }
}

fn prompt() -> String {
    // Print the prompt.
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input.
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    input
}

enum BuiltInCommand {
    Echo,
    Exit,
    Pwd,
    Type,
}

impl TryFrom<&str> for BuiltInCommand {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "echo" => Ok(BuiltInCommand::Echo),
            "exit" => Ok(BuiltInCommand::Exit),
            "pwd" => Ok(BuiltInCommand::Pwd),
            "type" => Ok(BuiltInCommand::Type),
            _ => Err(format!("Unknown builtin command {}", value)),
        }
    }
}

impl Display for BuiltInCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            BuiltInCommand::Echo => "echo",
            BuiltInCommand::Exit => "exit",
            BuiltInCommand::Pwd => "pwd",
            BuiltInCommand::Type => "type",
        };

        write!(f, "{}", val)
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    // Load the PATH env variable.
    let path = std::env::var("PATH").unwrap();
    let directories = path.split(':');

    // Check whether the file exists in any of the directories.
    directories
        .into_iter()
        .find_map(|dir| Some(Path::new(dir).join(name)).filter(|location| location.exists()))
}

fn type_of_binary(args: &str) {
    if let Some(location) = find_in_path(args) {
        println!("{} is {}", args, location.display());
    } else {
        eprintln!("{}: not found", args);
    }
}

fn run_binary(cmd: &str, args: Option<&str>) {
    if let Some(_) = find_in_path(cmd) {
        let mut command = Command::new(cmd);
        if let Some(args) = args {
            command.args(args.split_ascii_whitespace());
        }

        // Start the program in a thread and wait for it to finish.
        command.status().unwrap();
    } else {
        eprintln!("{}: command not found", cmd);
    }
}
