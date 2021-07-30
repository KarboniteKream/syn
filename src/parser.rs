use std::collections::{HashMap, HashSet, VecDeque};
use std::error;
use std::fmt::{self, Display, Formatter};

use crate::automaton::{Action, Automaton, Data, Item, Table};
use crate::grammar::{Grammar, Position, Symbol};
use crate::lexer::Token;
use crate::util;

/// Performs parsing using LLLR and returns the list of rules.
pub fn parse_lllr(tokens: &[Token], grammar: &mut Grammar) -> Result<Vec<usize>, Error> {
    let (parse_table, tables) = get_lllr_tables(grammar)?;

    let mut rules = Vec::new();
    let mut stack = vec![(Symbol::Start.id(), (0, 0))];
    let mut input = get_input(tokens);

    while !stack.is_empty() && !input.is_empty() {
        let (symbol, position) = *stack.last().unwrap();
        let token = input.front().cloned().unwrap();

        if let Some(&rule) = parse_table.get(&(symbol, token.symbol)) {
            rules.push(rule);
            stack.pop();

            for (idx, symbol) in grammar.rule(rule).body.iter().enumerate().rev() {
                if *symbol != Symbol::Null.id() {
                    stack.push((*symbol, (rule, idx)));
                }
            }

            continue;
        }

        // Start the LR parser if necessary.
        if let Some(data) = tables.get(&position) {
            let grammar = &data.grammar;

            let mut lr_rules = vec![Vec::new()];
            let mut lr_stack = vec![(symbol, 0)];
            input.push_front(Token::end());

            let is_valid = loop {
                if lr_stack.is_empty() {
                    break false;
                }

                let state = lr_stack.last().unwrap().1;
                let token = input.front().cloned().unwrap_or_else(Token::null);

                // Check if the LR parser can stop.
                if let Some((item, tail)) = find_unique_item(grammar, data, state, &token) {
                    reduce_rules(&mut lr_rules, item.dot);
                    lr_rules.last_mut().unwrap().insert(0, item.rule);

                    let count = lr_rules.len() - 1;
                    reduce_rules(&mut lr_rules, count);

                    // Replace embedded parser symbols on the LL stack.
                    let body = grammar.rule(data.start_rule).tail(1);
                    stack.truncate(stack.len() - body.len());
                    stack.extend(tail.iter().rev());

                    break true;
                }

                let action = match data.action_table.get(&(state, token.symbol)) {
                    Some(action) => action,
                    None => break false,
                };

                match action {
                    Action::Shift(state) => {
                        lr_stack.push((token.symbol, *state));
                        lr_rules.push(Vec::new());
                        input.pop_front();
                    }
                    Action::Reduce(rule) => {
                        let rule = grammar.rule(*rule);
                        reduce_stack(&mut lr_stack, &rule.body)?;
                        reduce_rules(&mut lr_rules, rule.body.len());
                        lr_rules.last_mut().unwrap().insert(0, rule.id);
                        lr_rules.push(Vec::new());

                        let state = lr_stack.last().unwrap().1;
                        let next_state = match data.goto_table.get(&(state, rule.head)) {
                            Some(&next_state) => next_state,
                            None => return Err(Error::Internal),
                        };

                        lr_stack.push((rule.head, next_state));
                    }
                    Action::Accept(rule) => {
                        let body = grammar.rule(*rule).tail(1);
                        reduce_stack(&mut lr_stack, body)?;
                        reduce_rules(&mut lr_rules, body.len());
                        stack.truncate(stack.len() - body.len());

                        break true;
                    }
                }
            };

            if !is_valid {
                return match next_token(&mut input, &grammar.symbols) {
                    Some(token) => Err(Error::Parse(token)),
                    None => Err(Error::Eof),
                };
            }

            rules.extend(lr_rules.last().unwrap());
            continue;
        }

        if symbol != token.symbol {
            break;
        }

        stack.pop();
        input.pop_front();
    }

    if !stack.is_empty() {
        return Err(Error::Eof);
    }

    if let Some(token) = next_token(&mut input, &grammar.symbols) {
        return Err(Error::Parse(token));
    }

    Ok(rules)
}

/// Performs parsing using LL(1) and returns the list of rules.
pub fn parse_ll(tokens: &[Token], grammar: &Grammar) -> Result<Vec<usize>, Error> {
    let parse_table = match get_ll_table(grammar, &HashSet::new()) {
        Ok(parse_table) => parse_table,
        Err(conflicts) => {
            let symbol = grammar.symbol(conflicts[0]);
            return Err(Error::Conflict(symbol.clone()));
        }
    };

    let mut rules = Vec::new();
    let mut stack = vec![Symbol::Start.id()];
    let mut input = get_input(tokens);

    while !stack.is_empty() && !input.is_empty() {
        let &symbol = stack.last().unwrap();
        let token = input.front().unwrap();

        if let Some(&rule) = parse_table.get(&(symbol, token.symbol)) {
            rules.push(rule);
            stack.pop();

            for &symbol in grammar.rule(rule).body.iter().rev() {
                if symbol != Symbol::Null.id() {
                    stack.push(symbol);
                }
            }

            continue;
        }

        if symbol != token.symbol {
            break;
        }

        stack.pop();
        input.pop_front();
    }

    if !stack.is_empty() {
        return Err(Error::Eof);
    }

    if let Some(token) = next_token(&mut input, &grammar.symbols) {
        return Err(Error::Parse(token));
    }

    Ok(rules)
}

/// Performs parsing using LR(1) and returns the list of rules.
pub fn parse_lr(tokens: &[Token], grammar: &Grammar, data: &Data) -> Result<Vec<usize>, Error> {
    let mut rules = Vec::new();
    let mut stack = vec![(Symbol::Start.id(), 0)];
    let mut input = get_input(tokens);

    let is_valid = loop {
        if stack.is_empty() {
            break false;
        }

        let state = stack.last().unwrap().1;
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
                reduce_stack(&mut stack, &rule.body)?;
                rules.push(rule.id);

                let state = stack.last().unwrap().1;
                let next_state = match data.goto_table.get(&(state, rule.head)) {
                    Some(&next_state) => next_state,
                    None => return Err(Error::Internal),
                };

                stack.push((rule.head, next_state));
            }
            Action::Accept(rule) => {
                let rule = grammar.rule(*rule);
                let mut body = rule.body.clone();
                body.insert(0, rule.head);

                reduce_stack(&mut stack, &body)?;
                rules.push(rule.id);

                break true;
            }
        }
    };

    if !stack.is_empty() {
        return Err(Error::Internal);
    }

    if !is_valid {
        return match next_token(&mut input, &grammar.symbols) {
            Some(token) => Err(Error::Parse(token)),
            None => Err(Error::Eof),
        };
    }

    rules.reverse();
    Ok(rules)
}

/// Constructs the LL parse table or returns the list of conflicts.
fn get_ll_table(
    grammar: &Grammar,
    ignored_symbols: &HashSet<usize>,
) -> Result<Table<usize>, Vec<usize>> {
    let mut parse_table = HashMap::new();
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
            if parse_table.insert((rule.head, symbol), rule.id).is_some() {
                conflicts.insert(rule.head);
            }
        }
    }

    if !conflicts.is_empty() {
        return Err(util::to_sorted_vec(conflicts));
    }

    Ok(parse_table)
}

/// Constructs the LL and embedded LR tables.
fn get_lllr_tables(
    grammar: &mut Grammar,
) -> Result<(Table<usize>, HashMap<Position, Data>), Error> {
    let parse_table = get_ll_table(grammar, &HashSet::new());

    let mut all_conflicts = HashSet::new();
    let mut wrappers = HashMap::new();

    // Find wrappers for conflicting symbols.
    if let Err(conflicts) = parse_table {
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

                    // Find a wrapper with a valid LR automaton.
                    let data = loop {
                        if tail.is_empty() {
                            break None;
                        }

                        symbols.push(tail.remove(0));
                        let follow = grammar.first_follow(&tail, rule.head);

                        if follow.is_empty() {
                            let symbol = grammar.symbol(symbol);
                            return Err(Error::Symbol(symbol.clone()));
                        }

                        let mut grammar = grammar.clone();
                        let rule = grammar.wrap_symbols(&symbols, &follow);

                        if let Ok(data) = Automaton::new(&grammar, rule).data() {
                            break Some(data);
                        }
                    };

                    if data.is_none() {
                        if all_conflicts.insert(rule.head) {
                            new_conflicts.insert(rule.head);
                        }

                        // Remove existing wrappers for the conflicting rule.
                        wrappers.remove(&rule.head);
                        break;
                    }

                    let wrapper = ((rule.id, idx), data.unwrap());
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

    // Construct the LL parse table, ignoring conflicting symbols.
    let parse_table = match get_ll_table(grammar, &all_conflicts) {
        Ok(parse_table) => parse_table,
        Err(_) => return Err(Error::Internal),
    };

    // Construct embedded LR tables for conflicting symbols.
    let tables: HashMap<Position, Data> = wrappers
        .values()
        .flatten()
        .map(|(position, data)| (*position, data.clone()))
        .collect();

    Ok((parse_table, tables))
}

/// Finds a unique item in the current automaton state.
/// Returns the item and the remaining symbols in the automaton.
fn find_unique_item(
    grammar: &Grammar,
    data: &Data,
    state: usize,
    token: &Token,
) -> Option<(Item, Vec<(usize, Position)>)> {
    let key = (state, token.symbol);

    if let Some(action) = data.action_table.get(&key) {
        if action.is_accept() {
            return None;
        }
    }

    if grammar.symbols[token.symbol].is_internal() {
        return None;
    }

    // Find the unique item.
    let mut from = match data.left_table.get(&key) {
        Some(&item) => (state, item),
        None => return None,
    };

    let item = data.items[&from.1];
    let rule = grammar.rule(item.rule);
    let mut tail = rule.positions(item.dot);
    let mut current_rule = rule.id;

    // Follow item transitions to find the remaining symbols.
    loop {
        from = match data.backtrack_table.get(&from) {
            Some(&to) => to,
            None => break,
        };

        let item = data.items[&from.1];
        let rule = grammar.rule(item.rule);

        if rule.id != current_rule {
            tail.extend(rule.positions(item.dot + 1));
            current_rule = rule.id;
        }
    }

    Some((item, tail))
}

/// Converts tokens to a parser input.
fn get_input(tokens: &[Token]) -> VecDeque<Token> {
    let mut input: VecDeque<Token> = tokens.iter().cloned().collect();

    input.push_front(Token::end());
    input.push_back(Token::end());

    input
}

/// Returns the next input token, ignoring internal symbols.
fn next_token(input: &mut VecDeque<Token>, symbols: &[Symbol]) -> Option<Token> {
    input
        .pop_front()
        .filter(|token| !symbols[token.symbol].is_internal())
}

/// Removes the rule symbols from the stack.
fn reduce_stack(stack: &mut Vec<(usize, usize)>, symbols: &[usize]) -> Result<(), Error> {
    for &id in symbols.iter().rev() {
        if id == Symbol::Null.id() {
            continue;
        }

        if stack.pop().filter(|&(symbol, _)| symbol == id).is_none() {
            return Err(Error::Internal);
        }
    }

    Ok(())
}

/// Merges the left-parse rules.
fn reduce_rules(rules: &mut Vec<Vec<usize>>, count: usize) {
    let mut buffer = Vec::new();

    for _ in 0..count {
        let rules = rules.pop().unwrap();
        buffer = [rules, buffer].concat();
    }

    rules.last_mut().unwrap().extend(buffer);
}

#[derive(Debug)]
pub enum Error {
    Conflict(Symbol),
    Eof,
    Internal,
    Parse(Token),
    Symbol(Symbol),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Conflict(symbol) => write!(f, "Conflict in table for {}", symbol),
            Self::Eof => write!(f, "Unexpected end of file"),
            Self::Internal => write!(f, "Internal error"),
            Self::Parse(token) => write!(f, "Unexpected token {}", token),
            Self::Symbol(symbol) => write!(f, "Cannot parse symbol {}", symbol),
        }
    }
}

impl error::Error for Error {}
