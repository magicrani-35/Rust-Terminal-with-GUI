use std::collections::HashMap;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

use super::builtins::BUILTINS;

pub(crate) struct ShellHelper {
    pub(crate) completions: HashMap<String, String>,
}

impl Highlighter for ShellHelper {}

impl Hinter for ShellHelper {
    type Hint = String;
}

impl Validator for ShellHelper {}

impl Helper for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>)
        -> rustyline::Result<(usize, Vec<Pair>)> {
        let before_cursor = &line[..pos];

        if let Some(space_position) = before_cursor.rfind(' ') {
            let start = space_position + 1;
            let prefix = &before_cursor[start..];

            if let Some(command) =
                before_cursor.split_whitespace().next() {
                if let Some(script) =
                    self.completions.get(command) {
                    let previous_word = before_cursor[..start]
                        .split_whitespace()
                        .last()
                        .unwrap_or("");

                    let output = match Command::new(script)
                        .arg(command)
                        .arg(prefix)
                        .arg(previous_word)
                        .env("COMP_LINE", line)
                        .env("COMP_POINT", pos.to_string())
                        .stdin(Stdio::null())
                        .output()
                    {
                        Ok(output) => output,
                        Err(_) => return Ok((start, Vec::new())),
                    };

                    let text = String::from_utf8_lossy(&output.stdout);

                    let names: Vec<&str> = text
                        .lines()
                        .filter(|line| !line.is_empty())
                        .collect();

                    let single_match = names.len() == 1;
                    let mut candidates = Vec::new();

                    for name in names {
                        candidates.push(Pair {
                            display: name.to_string(),
                            replacement: if single_match {
                                format!("{name} ")
                            } else {
                                name.to_string()
                            },
                        });
                    }

                    return Ok((start, candidates))
                }
            }

            return complete_filename(prefix, start);
        }

        let prefix = before_cursor;

        if prefix.is_empty() {
            return Ok((0, Vec::new()));
        }

        let mut names = std::collections::BTreeSet::new();

        for &builtin in BUILTINS {
            if builtin.starts_with(prefix) {
                names.insert(builtin.to_string());
            }
        }

        if let Some(path) = env::var_os("PATH") {
            for directory in env::split_paths(&path) {
                let directory = if directory.as_os_str().is_empty() {
                    std::path::PathBuf::from(".")
                } else {
                    directory
                };

                let entries = match std::fs::read_dir(directory) {
                    Ok(entries) => entries,
                    Err(_) => continue,
                };

                for entry in entries {
                    let entry = match entry {
                        Ok(name) => name,
                        Err(_) => continue,
                    };

                    let name = match entry.file_name().into_string() {
                        Ok(name) => name,
                        Err(_) => continue,
                    };

                    if !name.starts_with(prefix) {
                        continue;
                    }

                    let metadata = match entry.path().metadata()
                    {
                        Ok(metadata) => metadata,
                        Err(_) => continue,
                    };

                    let is_executable = metadata.permissions().mode() &0o111 != 0;

                    if metadata.is_file() && is_executable {
                        names.insert(name);
                    }
                }
            }
        }

        let single_match = names.len() == 1;
        let mut candidates = Vec::new();

        for name in names {
            let replacement = if single_match {
                format!("{name} ")
            } else {
                name.clone()
            };

            candidates.push(Pair {
                display: name,
                replacement,
            });
        }

        Ok((0, candidates))
    }
}

fn complete_filename(
    prefix: &str,
    start: usize,
) -> rustyline::Result<(usize, Vec<Pair>)> {
    let (directory_prefix, filename_prefix) = match prefix.rfind('/') {
        Some(position) => prefix.split_at(position + 1),
        None => ("", prefix),
    };

    let search_directory = if directory_prefix.is_empty() {
        "."
    } else {
        directory_prefix
    };

    let entries = match std::fs::read_dir(search_directory) {
        Ok(entries) => entries,
        Err(_) => return Ok((start, Vec::new())),
    };

    let mut names = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let name = match entry.file_name().into_string()
        {
            Ok(name) => name,
            Err(_) => continue,
        };

        if !name.starts_with(filename_prefix) {
            continue;
        }

        let path = entry.path();

        if path.is_dir() {
            names.push(format!("{directory_prefix}{name}/"));
        } else if path.is_file() {
            names.push(format!("{directory_prefix}{name}"));
        }
    }

    names.sort();

    let single_match = names.len() == 1;
    let mut candidates = Vec::new();

    for name in names {
        let replacement = if single_match && !name.ends_with('/') {
            format!("{name} ")
        } else {
            name.clone()
        };

        candidates.push(Pair {
            display: name,
            replacement,
        });
    }

    Ok((start, candidates))
}
