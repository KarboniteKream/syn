use std::collections::{HashMap, HashSet, VecDeque};
use std::error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

use crate::automaton::{Action, Data};
use crate::grammar::Grammar;
use crate::symbol::Symbol;
use crate::util::{self, Table};

mod span;
mod token;

use span::Span;
use token::Token;

/// Parses the input file and returns the list of rules.
pub fn parse_file(filename: &Path, grammar: &Grammar, data: &Data) -> Result<Vec<usize>, Error> {
    let tokens = get_tokens(filename, grammar)?;

    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    if let Ok(rules) = parse_ll(&tokens, grammar) {
        return Ok(rules);
    }

    parse_lr(&tokens, grammar, data)
}

/// Performs parsing using LL(1).
fn parse_ll(tokens: &[Token], grammar: &Grammar) -> Result<Vec<usize>, Error> {
    let table = match get_ll_table(grammar) {
        Ok(table) => table,
        Err(conflicts) => {
            let symbol = grammar.symbol(conflicts[0]);
            return Err(Error::Conflict(symbol.clone()));
        }
    };

    let mut input = get_input(tokens);
    let mut stack = vec![0];
    let mut rules = Vec::new();

    while !input.is_empty() && !stack.is_empty() {
        let &symbol = stack.last().unwrap();
        let token = input.front().unwrap();

        if let Some(&rule) = table.get(&(symbol, token.symbol)) {
            stack.pop();

            for &symbol in grammar.rule(rule).body.iter().rev() {
                if symbol != Symbol::Null.id() {
                    stack.push(symbol);
                }
            }

            if rule != 0 {
                rules.push(rule);
            }

            continue;
        }

        if symbol != token.symbol {
            break;
        }

        input.pop_front();
        stack.pop();
    }

    if let Some(token) = input.pop_front() {
        return Err(Error::Parse(token));
    }

    if !stack.is_empty() {
        return Err(Error::EOF);
    }

    Ok(rules)
}

/// Performs parsing using LR(1).
fn parse_lr(tokens: &[Token], grammar: &Grammar, data: &Data) -> Result<Vec<usize>, Error> {
    let mut input = get_input(tokens);
    let mut stack = vec![(0, 0)];
    let mut rules = Vec::new();

    let is_valid = loop {
        if input.is_empty() || stack.is_empty() {
            break false;
        }

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
                rules.push(rule.id);
            }
            Action::Accept => {
                break true;
            }
        }
    };

    if !is_valid {
        return match input.pop_front() {
            Some(token) => Err(Error::Parse(token)),
            None => Err(Error::EOF),
        };
    }

    Ok(rules)
}

/// Constructs the LL parsing table or returns the list of conflicts.
fn get_ll_table(grammar: &Grammar) -> Result<Table<usize>, Vec<usize>> {
    let mut table = HashMap::new();
    let mut conflicts = HashSet::new();

    for rule in &grammar.rules {
        let mut symbols = grammar.first_sequence(&rule.body);

        if symbols.contains(&Symbol::Null.id()) {
            symbols = grammar.follow(rule.head);
        }

        for symbol in symbols {
            if table.insert((rule.head, symbol), rule.id).is_some() {
                conflicts.insert(rule.head);
            }
        }
    }

    if !conflicts.is_empty() {
        return Err(util::to_sorted_vec(conflicts));
    }

    Ok(table)
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

/// Converts tokens to a parser input.
fn get_input(tokens: &[Token]) -> VecDeque<Token> {
    let mut input: VecDeque<Token> = tokens.iter().cloned().collect();

    let delimiter = Token::new(Symbol::End.id(), Symbol::End.name(), Span::default());
    input.push_front(delimiter.clone());
    input.push_back(delimiter);

    input
}

#[derive(Debug)]
pub enum Error {
    Conflict(Symbol),
    EOF,
    File(String),
    Internal,
    Parse(Token),
    Token(String, Span),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Conflict(symbol) => write!(f, "Conflict in table for {}", symbol),
            Error::EOF => write!(f, "Unexpected end of file"),
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
