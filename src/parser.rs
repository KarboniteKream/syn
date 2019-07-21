use std::collections::VecDeque;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

use crate::automaton::{Action, Data};
use crate::grammar::Grammar;
use crate::symbol::Symbol;

mod span;
mod token;

use span::Span;
use token::Token;

/// Parses the input file using a LR(1) parser and returns the list of tokens.
pub fn parse_file(filename: &Path, grammar: &Grammar, data: &Data) -> Result<Vec<Token>, Error> {
    let tokens = get_tokens(filename, grammar)?;

    if tokens.is_empty() {
        return Ok(tokens);
    }

    let mut input: VecDeque<Token> = tokens.iter().cloned().collect();
    let mut stack = vec![(0, 0)];
    let mut is_valid = false;

    let delimiter = Token::new(Symbol::End.id(), Symbol::End.name(), Span::default());
    input.push_front(delimiter.clone());
    input.push_back(delimiter);

    while !input.is_empty() && !stack.is_empty() {
        let &(_, state) = stack.last().unwrap();
        let token = input.front().unwrap();
        let action = data.action_table.get(&(state, token.symbol));

        if action.is_none() {
            return Err(Error::Parse(token.clone()));
        }

        match action.unwrap() {
            Action::Shift(state) => {
                stack.push((token.symbol, *state));
                input.pop_front();
            }
            Action::Reduce(rule) => {
                let rule = grammar.rule(*rule);

                for id in rule.body.iter().rev() {
                    if stack.pop().filter(|(symbol, _)| symbol == id).is_none() {
                        return Err(Error::Internal);
                    }
                }

                let &(_, state) = stack.last().unwrap();
                let state = match data.goto_table.get(&(state, rule.head)) {
                    Some(&state) => state,
                    None => return Err(Error::Internal),
                };

                stack.push((rule.head, state));
            }
            Action::Accept => {
                is_valid = true;
                break;
            }
        }
    }

    if !is_valid {
        return match input.pop_front() {
            Some(token) => Err(Error::Parse(token)),
            None => Err(Error::Internal),
        };
    }

    Ok(tokens)
}

/// Returns the list of tokens in the input file using lexical analysis.
fn get_tokens(filename: &Path, grammar: &Grammar) -> Result<Vec<Token>, Error> {
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
    Internal,
    Parse(Token),
    Token(String, Span),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::File(error) => write!(f, "Cannot read file {}", error),
            Error::Internal => write!(f, "Internal error"),
            Error::Parse(token) => write!(f, "Unexpected token {}", token),
            Error::Token(lexeme, span) => {
                let lexeme = lexeme.escape_default();
                write!(f, "Cannot recognize token '{}' @ {}", lexeme, span)
            }
        }
    }
}
impl error::Error for Error {}
