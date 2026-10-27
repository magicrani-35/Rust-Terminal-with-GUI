use crate::shell::jobs::report_jobs;
use crate::shell::completion::ShellHelper;
use crate::shell::builtins::run_builtin;
use crate::shell::{LineOutcome, run_line};

use std::collections::HashMap;
use std::env;
use std::path::Path;
use super::history;
use super::state::ShellState;

use rustyline::error::ReadlineError;
use rustyline::{Editor};
use rustyline::history::DefaultHistory;

pub(crate) fn run() -> rustyline::Result<()> {
    let startup_args: Vec<String> =
        env::args().skip(1).collect();

    if startup_args.first().map(String::as_str) == Some("--run-builtin") {
        let Some(command) = startup_args.get(1) else {
            eprintln!("expected builtin name");
            return Ok(());
        };

        let args: Vec<&str> = startup_args[2..]
            .iter()
            .map(String::as_str)
            .collect();

        let mut completions = HashMap::new();
        let mut state = ShellState::default();

        run_builtin(
            command,
            &args,
            &mut completions,
            &mut state,
        );

        return Ok(());
    }
    let config = rustyline::Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();

    let mut editor = Editor::<ShellHelper, DefaultHistory>::with_config(config)?;
    editor.set_helper(Some(ShellHelper {
        completions: HashMap::new(),
    }));



    let mut state = ShellState::default();
    let history_path = env::var_os("HISTFILE");

    if let Some(path) = &history_path {
        match history::load(Path::new(path)) {
            Ok(entries) => {
                for entry in entries {
                    editor.add_history_entry(entry.as_str())?;
                    state.command_history.push(entry);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // No existing history file to load.
            }
            Err(e) => {
                eprintln!("history: {e}");
            }
        }
    }

    // Loaded entries are already on disk. Append only entries added afterward.
    state.history_append_from = state.command_history.len();


    loop {
        report_jobs(&mut state.jobs, false);

        match editor.readline("$ ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                state.command_history.push(line.to_string());
                editor.add_history_entry(line)?;

                let outcome = {
                    let helper = editor
                        .helper_mut()
                        .expect("ShellHelper was installed");

                    run_line(
                        line,
                        &mut helper.completions,
                        &mut state,
                    )
                };

                if let LineOutcome::Exit = outcome {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C: cancel the current line, don't kill the shell.
                continue;
            }
            Err(ReadlineError::Eof) => break, // Ctrl+D
            Err(e) => {
                eprintln!("$: readline error: {e}");
                break;
            }
        }
    }

    if let Some(path) = &history_path {
        if let Err(e) = history::save(Path::new(path), &state.command_history) {
            eprintln!("history: {e}");
        }
    }


    Ok(())

}