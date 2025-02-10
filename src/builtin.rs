use crate::io_redirection::IoRedirections;
use crate::path::find_binary_in_path;

pub(crate) enum BuiltInCommand {
    ChangeDirectory,
    Echo,
    Exit,
    PrintWorkingDirectory,
    Type,
}

impl TryFrom<&str> for BuiltInCommand {
    type Error = String;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        match command {
            "cd" => Ok(BuiltInCommand::ChangeDirectory),
            "echo" => Ok(BuiltInCommand::Echo),
            "exit" => Ok(BuiltInCommand::Exit),
            "pwd" => Ok(BuiltInCommand::PrintWorkingDirectory),
            "type" => Ok(BuiltInCommand::Type),
            _ => Err(format!("Unknown builtin command {}", command)),
        }
    }
}

impl BuiltInCommand {
    fn name(&self) -> String {
        match self {
            BuiltInCommand::ChangeDirectory => "cd",
            BuiltInCommand::Echo => "echo",
            BuiltInCommand::Exit => "exit",
            BuiltInCommand::PrintWorkingDirectory => "pwd",
            BuiltInCommand::Type => "type",
        }
        .to_owned()
    }

    /// Runs the built-in command.
    ///
    /// # Note
    /// The run method doesn't accept a stderr argument as it doesn't write to the standard error
    /// under regular circumstances. It any error is encountered, they are returned as error types.
    pub(crate) fn run(
        &self,
        args: &[String],
        io_redirections: &mut IoRedirections,
    ) -> Result<(), String> {
        match self {
            BuiltInCommand::ChangeDirectory => {
                let arg = get_single_argument(args)?;

                let working_directory = if arg == "~" {
                    std::env::var("HOME")
                        .map_err(|e| format!("Invalid HOME environment variable: {:?}", e))?
                } else {
                    arg
                };

                std::env::set_current_dir(&working_directory)
                    .map_err(|_| format!("cd: {}: No such file or directory", working_directory))?;
            }
            BuiltInCommand::Echo => {
                io_redirections.writeln(format_args!("{}", args.join(" ")))?;
            }
            BuiltInCommand::Exit => {
                let arg = get_single_argument(args)?;

                let exit_code = arg
                    .parse::<i32>()
                    .map_err(|e| format!("Invalid exit code '{}': {:?}", arg, e))?;

                std::process::exit(exit_code);
            }
            BuiltInCommand::PrintWorkingDirectory => {
                let cwd = std::env::current_dir().map_err(|e| {
                    format!("Failed to determine the current working directory: {:?}", e)
                })?;

                io_redirections.writeln(format_args!("{}", &cwd.display()))?;
            }
            BuiltInCommand::Type => {
                let arg = get_single_argument(args)?;

                if let Ok(sub_command) = BuiltInCommand::try_from(arg.as_ref()) {
                    io_redirections
                        .writeln(format_args!("{} is a shell builtin", sub_command.name()))?;
                } else if let Some(location) = find_binary_in_path(&arg)? {
                    io_redirections.writeln(format_args!("{} is {}", arg, location.display()))?;
                } else {
                    return Err(format!("{}: not found", arg));
                }
            }
        };

        Ok(())
    }
}

fn get_single_argument(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        Err("Not enough arguments, expected exactly 1".to_owned())
    } else if 1 < args.len() {
        Err("Too many arguments".to_owned())
    } else {
        Ok(args[0].trim().to_owned())
    }
}
