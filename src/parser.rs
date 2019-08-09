use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::identity;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

use crate::automaton::{Action, Automaton, Data};
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

    if let Ok(rules) = parse_lllr(&tokens, grammar) {
        return Ok(rules);
    }

    if let Ok(rules) = parse_ll(&tokens, grammar) {
        return Ok(rules);
    }

    parse_lr(&tokens, grammar, data)
}

/// Performs parsing using LLLR.
fn parse_lllr(tokens: &[Token], grammar: &Grammar) -> Result<Vec<usize>, Error> {
    let parsing_table = get_parsing_table(grammar, &HashSet::new());

    let mut all_conflicts = HashSet::new();
    let mut wrappers = HashMap::new();

    // Find wrappers for conflicting symbols.
    if let Err(conflicts) = parsing_table {
        all_conflicts.extend(conflicts);
        let mut conflicts = all_conflicts.clone();

        while !conflicts.is_empty() {
            let mut new_conflicts = HashSet::new();

            for rule in &grammar.rules {
                // Ignore rules for conflicting symbols.
                if all_conflicts.contains(&rule.head) {
                    continue;
                }

                let mut idx = 0;

                while idx < rule.body.len() {
                    let symbol = rule.body[idx];

                    // Ignore non-conflicting symbols.
                    if !conflicts.contains(&symbol) {
                        idx += 1;
                        continue;
                    }

                    let mut symbols = Vec::new();
                    let mut tail = rule.tail(idx).to_vec();
                    let mut follow = Vec::new();

                    // Find a wrapper with a valid LR automaton.
                    let is_valid = loop {
                        if tail.is_empty() {
                            break false;
                        }

                        symbols.push(tail.remove(0));
                        follow = grammar.first_follow(&tail, rule.head);

                        let mut grammar = grammar.clone();
                        let rule = grammar.wrap_symbols(&symbols, &follow);

                        if Automaton::new(&grammar, rule).is_valid() {
                            break true;
                        }
                    };

                    if !is_valid {
                        if all_conflicts.insert(rule.head) {
                            new_conflicts.insert(rule.head);
                        }

                        // If the rule itself is conflicting,
                        // its wrappers are unnecessary.
                        wrappers.remove(&rule.head);
                        break;
                    }

                    let wrapper = (rule.id, idx, symbols.clone(), follow);
                    idx += symbols.len();

                    wrappers
                        .entry(rule.head)
                        .or_insert_with(Vec::new)
                        .push(wrapper);
                }
            }

            conflicts = new_conflicts;
        }
    }

    let mut grammar = grammar.clone();
    let mut tables = HashMap::new();

    // Wrap conflicting symbols.
    let wrappers: Vec<(usize, (usize, usize), usize)> =
        util::to_sorted_vec(wrappers.values().flat_map(identity).cloned())
            .into_iter()
            .map(|(id, idx, symbols, follow)| {
                let rule = grammar.wrap_symbols(&symbols, &follow);
                (id, (idx, idx + symbols.len()), rule)
            })
            .collect();

    // Apply wrappers in reverse order to avoid recalculating indices.
    for &(id, (from, to), rule) in wrappers.iter().rev() {
        let symbol = grammar.rule(rule).head;
        grammar.rules[id].body.splice(from..to, vec![symbol]);
    }

    // Construct embedded automatons.
    for (_, _, rule) in wrappers {
        let symbol = grammar.rule(rule).head;

        if tables.contains_key(&symbol) {
            continue;
        }

        match Automaton::new(&grammar, rule).data() {
            Ok(data) => tables.insert(symbol, data),
            Err(_) => return Err(Error::Internal),
        };
    }

    let parsing_table = match get_parsing_table(&grammar, &all_conflicts) {
        Ok(parsing_table) => parsing_table,
        Err(_) => return Err(Error::Internal),
    };

    let mut input = get_input(tokens);
    let mut stack = vec![Symbol::Start.id()];
    let mut rules = Vec::new();

    while !input.is_empty() && !stack.is_empty() {
        let &symbol = stack.last().unwrap();
        let token = input.front().unwrap();

        if let Some(&rule) = parsing_table.get(&(symbol, token.symbol)) {
            stack.pop();

            for &symbol in grammar.rule(rule).body.iter().rev() {
                if symbol != Symbol::Null.id() {
                    stack.push(symbol);
                }
            }

            rules.push(rule);
            continue;
        }

        if let Some(_data) = tables.get(&symbol) {
            return Err(Error::Internal);
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

/// Performs parsing using LL(1).
fn parse_ll(tokens: &[Token], grammar: &Grammar) -> Result<Vec<usize>, Error> {
    let parsing_table = match get_parsing_table(grammar, &HashSet::new()) {
        Ok(parsing_table) => parsing_table,
        Err(conflicts) => {
            let symbol = grammar.symbol(conflicts[0]);
            return Err(Error::Conflict(symbol.clone()));
        }
    };

    let mut input = get_input(tokens);
    let mut stack = vec![Symbol::Start.id()];
    let mut rules = Vec::new();

    while !input.is_empty() && !stack.is_empty() {
        let &symbol = stack.last().unwrap();
        let token = input.front().unwrap();

        if let Some(&rule) = parsing_table.get(&(symbol, token.symbol)) {
            stack.pop();

            for &symbol in grammar.rule(rule).body.iter().rev() {
                if symbol != Symbol::Null.id() {
                    stack.push(symbol);
                }
            }

            rules.push(rule);
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
    let mut stack = vec![(Symbol::Start.id(), 0)];
    let mut rules = Vec::new();

    let is_valid = loop {
        if stack.is_empty() {
            break false;
        }

        let &(_, state) = stack.last().unwrap();
        let token = input.front().cloned().unwrap_or_else(Token::null);
        let action = data.action_table.get(&(state, token.symbol));

        if action.is_none() {
            break false;
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

                rules.push(rule.id);
                stack.push((rule.head, state));
            }
            Action::Accept(rule) => {
                rules.push(*rule);
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

    rules.reverse();
    Ok(rules)
}

/// Constructs the LL parsing table or returns the list of conflicts.
fn get_parsing_table(
    grammar: &Grammar,
    ignored_symbols: &HashSet<usize>,
) -> Result<Table<usize>, Vec<usize>> {
    let mut parsing_table = HashMap::new();
    let mut conflicts = HashSet::new();

    for rule in &grammar.rules {
        if ignored_symbols.contains(&rule.head) {
            continue;
        }

        let mut symbols = grammar.first_sequence(&rule.body);

        if symbols.contains(&Symbol::Null.id()) {
            symbols = grammar.follow(rule.head);
        }

        for symbol in symbols {
            if parsing_table.insert((rule.head, symbol), rule.id).is_some() {
                conflicts.insert(rule.head);
            }
        }
    }

    if !conflicts.is_empty() {
        return Err(util::to_sorted_vec(conflicts));
    }

    Ok(parsing_table)
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

    input.push_front(Token::end());
    input.push_back(Token::end());

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
