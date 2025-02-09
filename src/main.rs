use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    loop {
        // Print the prompt and grab the input.
        let input = input_prompt();

        // Split the command and arguments.
        let (command, args) = input
            .split_once(' ')
            .map(|(cmd, args)| (cmd, Some(args)))
            .unwrap_or((&input, None));

        let _ = match BuiltInCommand::try_from(command) {
            Ok(command) => command.run(args),
            _ => run_binary(command, args),
        }
        .map_err(|e| eprintln!("{}", e));
    }
}

fn input_prompt() -> String {
    // Print the prompt.
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input.
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();

    input.trim().to_owned()
}

enum BuiltInCommand {
    Echo,
    Exit,
    Pwd,
    Type,
}

impl TryFrom<&str> for BuiltInCommand {
    type Error = String;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        match command {
            "echo" => Ok(BuiltInCommand::Echo),
            "exit" => Ok(BuiltInCommand::Exit),
            "pwd" => Ok(BuiltInCommand::Pwd),
            "type" => Ok(BuiltInCommand::Type),
            _ => Err(format!("Unknown builtin command {}", command)),
        }
    }
}

impl BuiltInCommand {
    fn name(&self) -> String {
        match self {
            BuiltInCommand::Echo => "echo",
            BuiltInCommand::Exit => "exit",
            BuiltInCommand::Pwd => "pwd",
            BuiltInCommand::Type => "type",
        }
        .to_owned()
    }

    fn run(&self, args: Option<&str>) -> Result<(), String> {
        match self {
            BuiltInCommand::Echo => {
                println!("{}", args.unwrap_or_default());
            }
            BuiltInCommand::Exit => {
                let exit_code = args.ok_or("Missing exit code".to_owned()).and_then(|s| {
                    s.parse::<i32>()
                        .map_err(|e| format!("Invalid exit code '{}': {:?}", s, e))
                })?;

                std::process::exit(exit_code);
            }
            BuiltInCommand::Pwd => {
                println!("{}", std::env::current_dir().unwrap().display());
            }
            BuiltInCommand::Type => {
                if let Some(args) = args {
                    match BuiltInCommand::try_from(args) {
                        Ok(arg_command) => {
                            println!("{} is a shell builtin", arg_command.name());
                        }
                        _ => type_of_binary(args)?,
                    }
                } else {
                    return Err("`type` expects a command name as argument".to_owned());
                }
            }
        };

        Ok(())
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

fn type_of_binary(args: &str) -> Result<(), String> {
    if let Some(location) = find_in_path(args) {
        println!("{} is {}", args, location.display());

        Ok(())
    } else {
        Err(format!("{}: not found", args))
    }
}

fn run_binary(cmd: &str, args: Option<&str>) -> Result<(), String> {
    if let Some(_) = find_in_path(cmd) {
        let mut command = Command::new(cmd);
        if let Some(args) = args {
            command.args(args.split_ascii_whitespace());
        }

        // Start the program in a thread and wait for it to finish.
        command.status().map(|_| {}).map_err(|e| format!("{:?}", e))
    } else {
        Err(format!("{}: command not found", cmd))
    }
}
