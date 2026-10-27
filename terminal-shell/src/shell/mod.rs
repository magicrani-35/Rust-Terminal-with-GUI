mod lexer;
mod parser;
mod variables;
mod jobs;
mod completion;
mod execution;
mod builtins;
pub(crate) mod session;
mod history;
mod state;

use std::collections::HashMap;

use self::builtins::{BuiltinResult, run_builtin};
use self::execution::{handle_redirection, run_external, run_pipeline, spawn_command};
use self::jobs::Job;
use self::lexer::{Token, tokenize};
use self::parser::parse_token;
use self::state::ShellState;

pub(crate) enum LineOutcome {
    Continue,
    Exit,
}

pub(super) fn run_line(
    line: &str,
    completions: &mut HashMap<String, String>,
    state: &mut ShellState,
) -> LineOutcome {
    let mut tokens = match tokenize(line, &state.variables) {
        Ok(tokens) => tokens,
        Err(message) => {
            eprintln!("$: {message}");
            return LineOutcome::Continue;
        }
    };

    let background = matches!(tokens.last(),
    Some(Token::Background));

    if background {
        tokens.pop();

        if tokens.is_empty() {
            eprintln!("expected command before &");
            return LineOutcome::Continue;
        }
    }

    if tokens.is_empty() {
        return LineOutcome::Continue;
    }

    let mut parsed_stages = Vec::new();

    for stage in tokens.split(|token| matches!(token, Token::Pipe)) {
        match parse_token(stage) {
            Ok(parsed) => parsed_stages.push(parsed),
            Err(message) => {
                eprintln!("$: {message}");
                return LineOutcome::Continue;
            }
        }
    }

    if background {
        if parsed_stages.len() != 1 {
            eprintln!("background pipelines are not supported yet");
            return LineOutcome::Continue;
        }

        let parsed = &parsed_stages[0];

        match spawn_command(parsed) {
            Ok(child) => {
                let id = state.jobs.iter().map(|job| job.id).max().unwrap_or(0) + 1;
                let pid = child.id();

                state.jobs.push(Job {
                    id,
                    child,
                    command: line
                        .trim_end()
                        .strip_suffix('&')
                        .unwrap_or(line)
                        .trim_end()
                        .to_string(),
                });

                println!("[{id}] {pid}");
            }
            Err(e) => {
                eprintln!("{}: {e}", parsed.command);
            }
        }

        return LineOutcome::Continue;
    }

    if parsed_stages.len() == 1 {
        let parsed = &parsed_stages[0];

        match handle_redirection(parsed) {
            Ok(true) => return LineOutcome::Continue,
            Ok(false) => {}
            Err(e) => {
                eprintln!("$: {e}");
                return LineOutcome::Continue;
            }
        }

        match run_builtin(
            parsed.command,
            &parsed.args,
            completions,
            state,
        )
        {
            BuiltinResult::Exit => return LineOutcome::Exit,
            BuiltinResult::Handled => return LineOutcome::Continue,
            BuiltinResult::NotBuiltin => {}
        }

        run_external(parsed.command, &parsed.args);
    } else {
        run_pipeline(&parsed_stages);
    }

    LineOutcome::Continue
}