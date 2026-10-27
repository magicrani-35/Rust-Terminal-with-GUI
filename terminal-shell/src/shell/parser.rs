use super::lexer::Token;

pub(crate) struct ParsedCommand<'a> {
    pub(crate) command: &'a str,
    pub(crate) args: Vec<&'a str>,
    pub(crate) input_file: Option<&'a str>,
    pub(crate) output_redirect: Option<(&'a str, &'a str)>,
    pub(crate) error_redirect: Option<(&'a str, &'a str)>,
}

pub(crate) fn parse_token<'a>(
    tokens: &'a [Token],
) -> Result<ParsedCommand<'a>, String> {
    let mut tokens = tokens.iter();
    let mut error_redirect = None;

    let command = match tokens.next() {
        Some(Token::Word(word)) => word.as_str(),
        _ => return Err(String::from("expected command")),
    };

    let mut args = Vec::new();
    let mut input_file = None;
    let mut output_redirect = None;

    while let Some(token) = tokens.next() {
        match token {
            Token::Word(word) => {
                args.push(word.as_str());
            }
            Token::Input | Token::Output | Token::Append | Token::ErrorOutput | Token::ErrorAppend =>
                {
                    let operator = match token {
                        Token::Input => "<",
                        Token::Output => ">",
                        Token::Append => ">>",
                        Token::ErrorOutput => "2>",
                        Token::ErrorAppend => "2>>",
                        _ => unreachable!(),
                    };

                    let filename = match tokens.next() {
                        Some(Token::Word(word)) => word.as_str(),
                        _ => {
                            return Err(format!(
                                "expected filename after {operator}"
                            ));
                        }
                    };

                    match operator {
                        "<" => input_file = Some(filename),
                        "2>" | "2>>" => {
                            error_redirect = Some((operator, filename));
                        }
                        _ => {
                            output_redirect = Some((operator, filename));
                        }
                    }
                }
            Token::Pipe => {
                return Err(String::from("unexpected pipe inside a command"));
            }
            Token::Background => {
                return Err(String::from(
                    "unexpected & inside a command"
                ));
            }
        }
    }

    Ok(ParsedCommand {
        command,
        args,
        input_file,
        output_redirect,
        error_redirect,
    })
}
