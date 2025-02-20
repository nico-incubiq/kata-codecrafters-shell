[![progress-banner](https://backend.codecrafters.io/progress/shell/1423bb98-cd78-4fa5-8cea-c6a1a63c5df5)](https://app.codecrafters.io/users/nico-incubiq)

# Challenge
This is a Rust solution to the
["Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview).

In this challenge, you'll build your own POSIX compliant shell that's capable of
interpreting shell commands, running external programs and builtin commands like
cd, pwd, echo and more. Along the way, you'll learn about shell command parsing,
REPLs, builtin commands, and more.

# Functionalities
## Basics
- `echo`: Print a message
- `exit`: Exit the shell
- `type`: Print information about an executable
- Run a program within the `$PATH`

## Navigation
- `cd`: Change the current working directory
- `pwd`: Print the current working directory

## Quoting
- Single-quotes, with escaping
- Double-quotes, with escaping (no variable expansion)

## Redirection
- Stdout and Stderr redirection
- Overriding with `>` and appending with `>>`

## Autocompletion
- Built-in commands
- `$PATH` executables
- Ring the terminal bell when no completion available
- Partial completions when multiple completions share a prefix
- Multi-completion by double-pressing TAB
