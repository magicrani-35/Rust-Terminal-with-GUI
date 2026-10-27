use std::env;
use std::fs::File;
use std::process::{Child, Command, Stdio};

use super::builtins::BUILTINS;
use super::parser::ParsedCommand;


pub(crate) fn run_pipeline(parsed_stages: &[ParsedCommand<'_>]) {
    let mut children: Vec<Child> = Vec::new();
    let mut previous_stdout: Option<Stdio> = None;

    for (i, parsed) in parsed_stages.iter().enumerate() {
        let mut stderr = Stdio::inherit();
        let is_last = i == parsed_stages.len() - 1;


        let mut stdin = if i == 0 {
            Stdio::inherit()
        } else {
            previous_stdout.take().unwrap_or_else(Stdio::null)
        };

        let mut stdout = if is_last { Stdio::inherit() } else {
            Stdio::piped()
        };

        if let Some(filename) = parsed.input_file {
            match File::open(filename) {
                Ok(file) => stdin = Stdio::from(file),
                Err(e) => {
                    eprintln!("$: {filename}: {e}");
                    break;
                }
            }
        }

        if let Some((operator, filename)) = parsed.output_redirect {
            let file_result = if operator == ">>" {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(filename)
            } else {
                File::create(filename)
            };
            match file_result {
                Ok(file) => stdout = Stdio::from(file),
                Err(e) => {
                    eprintln!("{filename}: {e}");
                    break;
                }
            }
        }

        if let Some((operator, filename)) = parsed.error_redirect {
            let file_result = if operator == "2>>" {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(filename)
            } else {
                File::create(filename)
            };
            match file_result {
                Ok(file) => stderr = Stdio::from(file),
                Err(e) => {
                    eprintln!("{filename}: {e}");
                    break;
                }
            }
        }

        let mut process = if BUILTINS.contains(&parsed.command) {
            let shell_path = match env::current_exe() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("cannot locate shell executable: {e}");
                    break;
                }
            };

            let mut child_shell = Command::new(shell_path);
            child_shell
                .arg("--run-builtin")
                .arg(parsed.command);

            child_shell
        } else {
            Command::new(parsed.command)
        };

        match process
            .args(&parsed.args)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()

        {
            Ok(mut child) => {
                previous_stdout = child.stdout.take().map(Stdio::from);
                children.push(child);
            }
            Err(e) => {
                eprintln!("$: {}: {e}", parsed.command);
                break;
            }
        }
    }

    // Start every stage before waiting so pipe readers and writers
    // can run concurrently, even when output exceeds the pipe's buffer.
    for mut child in children {
        let _ = child.wait();
    }
}


pub(crate) fn spawn_command(
    parsed: &ParsedCommand<'_>,
) -> std::io::Result<Child> {
    let mut process = Command::new(parsed.command);
    process.args(&parsed.args);

    if let Some(filename) = parsed.input_file {
        let file = File::open(filename)?;
        process.stdin(Stdio::from(file));
    }

    if let Some((operator, filename)) =
        parsed.output_redirect {
        let file = if operator == ">>" {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(filename)?
        } else {
            File::create(filename)?
        };

        process.stdout(Stdio::from(file));
    }

    if let Some((operator, filename)) = parsed.error_redirect {
        let file = if operator == "2>>" {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(filename)?
        } else {
            File::create(filename)?
        };

        process.stderr(Stdio::from(file));
    }

    process.spawn()
}

pub(crate) fn handle_redirection(
    parsed: &ParsedCommand<'_>,
) -> std::io::Result<bool> {
    if parsed.input_file.is_none()
        && parsed.output_redirect.is_none()
        && parsed.error_redirect.is_none()
    {
        return Ok(false);
    }

    spawn_command(parsed)?.wait()?;

    Ok(true)
}

pub(crate) fn run_external(command: &str, args: &[&str]) {
    match Command::new(command).args(args).spawn() {
        Ok(mut child) => {
            if let Err(e) = child.wait() {
                eprintln!("$: failed to wait on {command}: {e}");
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("{command}: command not found");
            } else {
                eprintln!("{command}: {e}");
            }
        }
    }
}