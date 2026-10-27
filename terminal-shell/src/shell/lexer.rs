use std::collections::HashMap;
use crate::shell::variables::is_valid_identifier;

#[derive(Debug)]
pub(crate) enum Token {
    Word(String),
    Pipe,
    Input,
    Output,
    Append,
    ErrorOutput,
    ErrorAppend,
    Background,
}

pub(crate) fn tokenize(
    line: &str,
    variables: &HashMap<String, String>,
) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    // Track whether a word exists separately from its contents:
    // an empty quoted string must still produce an argument.
    let mut token_started = false;

    let mut characters = line.chars().peekable();

    while let Some(character) = characters.next() {
        match character {
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                token_started = true;
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                token_started = true;
            }
            '\\' if !in_single_quotes => {
                if in_double_quotes {
                    match characters.peek().copied() {
                        Some(next) if matches!(next, '"' | '\\' | '$' | '`') => {
                            characters.next();
                            current.push(next);
                            token_started = true;
                        }
                        Some('\n') => {
                            characters.next();
                        }
                        _ => {
                            current.push('\\');
                            token_started = true;
                        }
                    }
                } else {
                    match characters.next() {
                        Some('\n') => {}
                        Some(next) => {
                            current.push(next);
                            token_started = true;
                        }
                        None => {
                            return Err(String::from(
                                "expected character after backslash"
                            ));
                        }
                    }
                }
            }
            '1' if !in_single_quotes
                && !in_double_quotes
                && !token_started
                && characters.peek() == Some(&'>') =>
                {
                    characters.next(); // continue the first >.

                    if characters.peek() == Some(&'>') {
                        characters.next(); // Consume the second >.
                        tokens.push(Token::Append);
                    } else {
                        tokens.push(Token::Output);
                    }
                }
            '2' if !in_single_quotes
                && !in_double_quotes
                && !token_started
                && characters.peek() == Some(&'>') =>
                {
                    characters.next();

                    if characters.peek() == Some(&'>') {
                        characters.next();
                        tokens.push(Token::ErrorAppend);
                    } else {
                        tokens.push(Token::ErrorOutput);
                    }
                }
            c if c.is_whitespace() && !in_double_quotes && !in_single_quotes => {
                if token_started {
                    tokens.push(Token::Word(std::mem::take(&mut current)));

                    token_started = false;
                }
            }
            '<' | '>' | '|' | '&' if !in_double_quotes && !in_single_quotes => {
                if token_started {
                    tokens.push(Token::Word(std::mem::take(&mut current)));
                    token_started = false;
                }
                let operator = match character {
                    '<' => Token::Input,
                    '|' => Token::Pipe,
                    '>' => {
                        if characters.peek() == Some(&'>') {
                            characters.next();
                            Token::Append
                        } else {
                            Token::Output
                        }
                    }
                    '&' => Token::Background,
                    _ => unreachable!(),
                };
                tokens.push(operator);
            }
            '$' if !in_single_quotes => {
                let mut name = String::new();

                if characters.peek() == Some(&'{') {
                    characters.next(); // Consume the opening brace.

                    let mut closed = false;

                    while let Some(next) = characters.next() {
                        if next == '}' {
                            closed = true;
                            break;
                        }

                        name.push(next);
                    }

                    if !closed {
                        return Err(String::from("unclosed parameter expansion"));
                    }

                    if !is_valid_identifier(&name) {
                        return Err(format!("${{{name}}}: bad substitution"));
                    }

                    if let Some(value) = variables.get(&name) {
                        current.push_str(value);
                    }
                } else {
                    // Existing $NAME behavior.
                    if let Some(next) = characters.peek().copied() {
                        if next.is_ascii_alphabetic() || next == '_' {
                            name.push(next);
                            characters.next();

                            while let Some(next) = characters.peek().copied() {
                                if next.is_ascii_alphanumeric() || next == '_' {
                                    name.push(next);
                                    characters.next();
                                } else {
                                    break;
                                }
                            }
                        }
                    }


                    if name.is_empty() {
                        current.push('$');
                    } else if let Some(value) = variables.get(&name) {
                        current.push_str(value);
                    }

                    token_started = true;
                }
            }

            c => {
                current.push(c);
                token_started = true;
            }
        }
    }

    if in_single_quotes {
        return Err(String::from("unclosed single quote"));
    }

    if in_double_quotes {
        return Err(String::from("unclosed double quote"));
    }

    if token_started {
        tokens.push(Token::Word(current));
    }

    Ok(tokens)
}
