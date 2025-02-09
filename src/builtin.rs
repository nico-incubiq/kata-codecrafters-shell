use crate::print_type_of_binary;

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

    pub(crate) fn run(&self, args: Option<&str>) -> Result<(), String> {
        match self {
            BuiltInCommand::ChangeDirectory => {
                let arg = args.ok_or("Missing working directory argument")?.trim();
                let working_directory = if arg == "~" {
                    &std::env::var("HOME")
                        .map_err(|e| format!("Invalid HOME environment variable: {:?}", e))?
                } else {
                    arg
                };

                std::env::set_current_dir(working_directory)
                    .map_err(|_| format!("cd: {}: No such file or directory", working_directory))?;
            }
            BuiltInCommand::Echo => {
                println!("{}", args.unwrap_or_default());
            }
            BuiltInCommand::Exit => {
                let exit_code = args
                    .ok_or("Missing exit code argument".to_owned())
                    .and_then(|s| {
                        s.parse::<i32>()
                            .map_err(|e| format!("Invalid exit code '{}': {:?}", s, e))
                    })?;

                std::process::exit(exit_code);
            }
            BuiltInCommand::PrintWorkingDirectory => {
                let cwd = std::env::current_dir().map_err(|e| {
                    format!("Failed to determine the current working directory: {:?}", e)
                })?;

                println!("{}", cwd.display());
            }
            BuiltInCommand::Type => {
                if let Some(args) = args {
                    match BuiltInCommand::try_from(args) {
                        Ok(arg_command) => {
                            println!("{} is a shell builtin", arg_command.name());
                        }
                        _ => print_type_of_binary(args)?,
                    }
                } else {
                    return Err("`type` expects a command name as argument".to_owned());
                }
            }
        };

        Ok(())
    }
}
