use std::collections::HashMap;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::jobs::report_jobs;
use super::variables::is_valid_identifier;
use super::history;
use super::ShellState;

pub(crate) enum BuiltinResult {
    NotBuiltin,
    Handled,
    Exit,
}

pub(crate) const BUILTINS: &[&str] = &[
    "exit",
    "echo",
    "type",
    "pwd",
    "cd",
    "complete",
    "jobs",
    "history",
    "declare",
];

fn find_executable(name: &str) -> Option<std::path::PathBuf> {
    let path = env::var_os("PATH")?;

    for directory in env::split_paths(&path) {
        let candidate = directory.join(name);

        if let Ok(metadata) = candidate.metadata() {
            let is_executable = metadata.permissions().mode() & 0o111 != 0;

            if metadata.is_file() && is_executable {
                return Some(candidate);
            }
        }
    }

    None
}


pub(super) fn run_builtin(
    command: &str,
    args: &[&str],
    completions: &mut HashMap<String, String>,
    state: &mut ShellState,
) -> BuiltinResult {
    let ShellState {
        jobs,
        command_history,
        history_append_from,
        variables,
    } = state;

    match command {
        "exit" => BuiltinResult::Exit,
        "cd" => {
            let target = match args.first().copied() {
                None | Some("~") => {
                    match env::var("HOME") {
                        Ok(home) => home,
                        Err(_) => {
                            eprintln!("cd: HOME not set");
                            return BuiltinResult::Handled;
                        }
                    }
                }
                Some(path) => path.to_string(),
            };

            if let Err(e) = env::set_current_dir(&target) {
                if e.kind() == std::io::ErrorKind::NotFound {
                    eprintln!("cd: {target}: No such file or directory");
                } else {
                    eprintln!("cd: {target}: {e}");
                }
            }
            BuiltinResult::Handled
        }
        "type" => {
            for &name in args {
                if BUILTINS.contains(&name) {
                    println!("{name} is a shell builtin");
                } else {
                    match find_executable(name) {
                        Some(path) => {
                            println!("{name} is {}", path.display());
                        }
                        None => {
                            println!("{name}: not found");
                        }
                    }
                }
            }
            BuiltinResult::Handled
        }
        "echo" => {
            println!("{}", args.join(" "));
            BuiltinResult::Handled
        }
        "pwd" => {
            match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => eprintln!("pwd: {e}"),
            }

            BuiltinResult::Handled
        }
        "complete" => {
            match args.first().copied() {
                Some("-C") => {
                    if let (Some(&script), Some(&name)) = (args.get(1), args.get(2))
                    {
                        completions.insert(
                            name.to_string(),
                            script.to_string(),
                        );
                    } else {
                        eprintln!("complete: expected -C <script> <command>");
                    }
                }
                Some("-p") => {
                    for &name in &args[1..] {
                        match completions.get(name) {
                            Some(script) => {
                                println!("complete -C '{script}' {name}");
                            }
                            None => {
                                eprintln!("complete: {name}: no completion specification");
                            }
                        }
                    }
                }
                Some("-r") => {
                    for &name in &args[1..] {
                        completions.remove(name);
                    }
                }
                _ => {}
            }

            BuiltinResult::Handled
        }
        "jobs" => {
            report_jobs(jobs, true);
            BuiltinResult::Handled
        }
        "history" => {
            if args.first().copied() == Some("-r") {
                let Some(&path) = args.get(1) else {
                    eprintln!("history: -r: expected a file path");
                    return BuiltinResult::Handled;
                };

                match history::load(Path::new(path)) {
                    Ok(entries) => {
                        command_history.extend(entries);
                    }
                    Err(e) => {
                        eprintln!("{path}: {e}");
                    }
                }
                return BuiltinResult::Handled;
            }

            if args.first().copied() == Some("-w") {
                let Some(&path) = args.get(1) else {
                    eprintln!("history: -w: expected a file path");
                    return BuiltinResult::Handled;
                };


                let mut contents = command_history.join("\n");

                if !command_history.is_empty() {
                    contents.push('\n');
                }

                if let Err(e) = history::save(Path::new(path), command_history) {
                    eprintln!("history: {path}: {e}");
                }

                return BuiltinResult::Handled;
            }
            if args.first().copied() == Some("-a") {
                let Some(&path) = args.get(1) else {
                    eprintln!("history: -a: expected a file path");
                    return BuiltinResult::Handled;
                };

                let new_entries = &command_history[*history_append_from..];

                match history::append(Path::new(path), new_entries) {
                    Ok(()) => {
                        *history_append_from = command_history.len();
                    }
                    Err(e) => {
                        eprintln!("history: {path}: {e}");
                    }
                }

                return BuiltinResult::Handled;
            }

            let count = match args.first() {
                Some(value) => match value.parse::<usize>() {
                    Ok(count) => count,
                    Err(_) => {
                        eprintln!("{value}: expected a non-negative integer");
                        return BuiltinResult::Handled
                    }
                },
                None => command_history.len(),
            };

            let start =
                command_history.len().saturating_sub(count);

            for (index, entry) in command_history
                .iter()
                .enumerate()
                .skip(start)

            {
                println!("{:>5}  {}", index + 1, entry);
            }

            BuiltinResult::Handled
        }
        "declare" => {
            if args.first().copied() == Some("-p") {
                for &name in &args[1..] {
                    match variables.get(name) {
                        Some(value) => {
                            println!("declare -- {name}=\"{value}\"");
                        }
                        None => {
                            eprintln!("declare: {name}: not found");
                        }
                    }
                }
            } else {
                for &assignment in args {
                    if let Some((name, value)) = assignment.split_once('=') {
                        if !is_valid_identifier(name) {
                            eprintln!("declare: `{assignment}': not a valid identifier");
                            continue;
                        }

                        variables.insert(
                            name.to_string(),
                            value.to_string());
                    }
                }
            }

            BuiltinResult::Handled
        }
        _ => BuiltinResult::NotBuiltin,
    }
}