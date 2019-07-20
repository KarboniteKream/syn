use std::error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

use crate::grammar::Grammar;
use crate::symbol::Symbol;

mod span;
mod token;

use span::Span;
use token::Token;

/// Returns the list of tokens in the input file using lexical analysis.
pub fn parse_file(filename: &Path, grammar: &Grammar) -> Result<Vec<Token>, Error> {
    let source: Vec<char> = match fs::read_to_string(filename) {
        Ok(contents) => contents.chars().collect(),
        Err(error) => return Err(Error::File(error.to_string())),
    };

    let mut idx = 0;
    let mut position = (1, 1);

    let mut current_match;
    let mut text = String::new();
    let mut span = Span::new(position);

    let mut last_token: Option<(Token, usize)> = None;
    let mut tokens = Vec::new();

    while idx < source.len() {
        let ch = source[idx];

        text.push(ch);
        current_match = grammar.find_symbol(&text);

        // There should always be at least a partial match.
        if current_match.is_none() && last_token.is_none() {
            return Err(Error::Token(text, span));
        }

        // If there's no match for the current string, take the last match.
        if current_match.is_none() {
            let (token, end_idx) = last_token.unwrap();

            // Ignore ϵ symbols.
            if token.symbol != Symbol::Null.id() {
                tokens.push(token.clone());
            }

            // Seek back to the end of the last match.
            let ch = token.last().unwrap();
            position = advance(token.span.end, ch);
            idx = end_idx + 1;

            text = String::new();
            span = Span::new(position);
            last_token = None;

            continue;
        }

        // Save the current full match.
        if let Some((id, true)) = current_match {
            let token = Token::new(id, text.clone(), span);
            last_token = Some((token, idx));
        }

        position = advance(position, ch);
        span.end = position;
        idx += 1;
    }

    Ok(tokens)
}

/// Advances the position in the file based on the current character.
fn advance(position: (usize, usize), ch: char) -> (usize, usize) {
    let (row, column) = position;

    if ch == '\n' {
        return (row + 1, 1);
    }

    (row, column + 1)
}

#[derive(Debug)]
pub enum Error {
    File(String),
    Token(String, Span),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::File(error) => write!(f, "Cannot read file {}", error),
            Error::Token(text, span) => {
                let text = text.escape_default();
                write!(f, "Cannot parse token '{}' @ {}", text, span)
            }
        }
    }
}
impl error::Error for Error {}
