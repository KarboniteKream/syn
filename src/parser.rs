use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::identity;
use std::error;
use std::fmt::{self, Display, Formatter};

use crate::automaton::{Action, Automaton, Data};
use crate::grammar::{Grammar, Symbol};
use crate::lexer::Token;
use crate::util::{self, Table};

/// Performs parsing using LLLR and returns the list of rules.
pub fn parse_lllr(tokens: &[Token], grammar: &mut Grammar) -> Result<Vec<usize>, Error> {
    let (parsing_table, tables) = get_lllr_parsing_tables(grammar)?;

    let mut input = get_input(tokens);
    let mut stack = vec![(Symbol::Start.id(), 0)];
    let mut rules = Vec::new();

    while !input.is_empty() && !stack.is_empty() {
        let symbol = stack.last().unwrap().0;
        let token = input.front().cloned().unwrap();

        if let Some(&rule) = parsing_table.get(&(symbol, token.symbol)) {
            stack.pop();

            for &symbol in grammar.rule(rule).body.iter().rev() {
                if symbol != Symbol::Null.id() {
                    stack.push((symbol, 0));
                }
            }

            rules.push(rule);
            continue;
        }

        // Start the LR parser if necessary.
        if let Some(data) = tables.get(&symbol) {
            input.push_front(Token::end());
            let mut lr_stack = stack.clone();
            let mut lr_rules = Vec::new();

            let is_valid = loop {
                if lr_stack.is_empty() {
                    break false;
                }

                let state = lr_stack.last().unwrap().1;
                let token = input.front().cloned().unwrap_or_else(Token::null);

                // Check if the LR parser can stop.
                if let Some((rule, tail)) = find_unique_rule(grammar, data, state, &token) {
                    let accept_rule = grammar.rules(symbol)[0].id;

                    lr_rules.push(rule);
                    stack.pop();

                    if accept_rule != rule {
                        lr_rules.push(accept_rule);
                    }

                    // Add the remaining symbols to the LL stack.
                    for &symbol in tail.iter().rev() {
                        stack.push((symbol, 0));
                    }

                    break true;
                }

                let action = match data.action_table.get(&(state, token.symbol)) {
                    Some(action) => action,
                    None => break false,
                };

                match action {
                    Action::Shift(state) => {
                        lr_stack.push((token.symbol, *state));
                        input.pop_front();
                    }
                    Action::Reduce(rule) => {
                        let rule = grammar.rule(*rule);

                        // Ignore the starting $ symbol in wrapper rules.
                        let mut body = rule.body.clone();
                        if tables.contains_key(&rule.head) {
                            body.remove(0);
                        }

                        reduce_rule(&mut lr_stack, &body)?;

                        let state = lr_stack.last().unwrap().1;
                        let next_state = match data.goto_table.get(&(state, rule.head)) {
                            Some(&next_state) => next_state,
                            None => return Err(Error::Internal),
                        };

                        lr_rules.push(rule.id);
                        lr_stack.push((rule.head, next_state));
                    }
                    Action::Accept(rule) => {
                        let rule = grammar.rule(*rule);

                        let mut body = rule.body.clone();
                        body.insert(0, rule.head);
                        reduce_rule(&mut lr_stack, &body)?;

                        lr_rules.push(rule.id);
                        break true;
                    }
                }
            };

            if !is_valid {
                return match next_token(&mut input, &grammar.symbols) {
                    Some(token) => Err(Error::Parse(token)),
                    None => Err(Error::EOF),
                };
            }

            lr_rules.reverse();
            rules.extend(lr_rules);

            continue;
        }

        if symbol != token.symbol {
            break;
        }

        input.pop_front();
        stack.pop();
    }

    if let Some(token) = next_token(&mut input, &grammar.symbols) {
        return Err(Error::Parse(token));
    }

    if !stack.is_empty() {
        return Err(Error::EOF);
    }

    Ok(rules)
}

/// Performs parsing using LL(1) and returns the list of rules.
pub fn parse_ll(tokens: &[Token], grammar: &Grammar) -> Result<Vec<usize>, Error> {
    let parsing_table = match get_ll_parsing_table(grammar, &HashSet::new()) {
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

    if let Some(token) = next_token(&mut input, &grammar.symbols) {
        return Err(Error::Parse(token));
    }

    if !stack.is_empty() {
        return Err(Error::EOF);
    }

    Ok(rules)
}

/// Performs parsing using LR(1) and returns the list of rules.
pub fn parse_lr(tokens: &[Token], grammar: &Grammar, data: &Data) -> Result<Vec<usize>, Error> {
    let mut input = get_input(tokens);
    let mut stack = vec![(Symbol::Start.id(), 0)];
    let mut rules = Vec::new();

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
                reduce_rule(&mut stack, &rule.body)?;

                let state = stack.last().unwrap().1;
                let next_state = match data.goto_table.get(&(state, rule.head)) {
                    Some(&next_state) => next_state,
                    None => return Err(Error::Internal),
                };

                rules.push(rule.id);
                stack.push((rule.head, next_state));
            }
            Action::Accept(rule) => {
                let rule = grammar.rule(*rule);

                let mut body = rule.body.clone();
                body.insert(0, rule.head);
                reduce_rule(&mut stack, &body)?;

                rules.push(rule.id);
                break true;
            }
        }
    };

    if !is_valid {
        return match next_token(&mut input, &grammar.symbols) {
            Some(token) => Err(Error::Parse(token)),
            None => Err(Error::EOF),
        };
    }

    if !stack.is_empty() {
        return Err(Error::Internal);
    }

    rules.reverse();
    Ok(rules)
}

/// Constructs the LL parsing table or returns the list of conflicts.
fn get_ll_parsing_table(
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

/// Constructs the LL and embedded LR parsing tables.
fn get_lllr_parsing_tables(
    grammar: &mut Grammar,
) -> Result<(Table<usize>, HashMap<usize, Data>), Error> {
    let parsing_table = get_ll_parsing_table(grammar, &HashSet::new());

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

    let mut tables = HashMap::new();

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

    let parsing_table = match get_ll_parsing_table(&grammar, &all_conflicts) {
        Ok(parsing_table) => parsing_table,
        Err(_) => return Err(Error::Internal),
    };

    Ok((parsing_table, tables))
}

/// Finds a unique item in the current automaton state.
/// Returns the rule the item represents and the remaining symbols in the automaton.
fn find_unique_rule(
    grammar: &Grammar,
    data: &Data,
    state: usize,
    token: &Token,
) -> Option<(usize, Vec<usize>)> {
    if grammar.symbols[token.symbol].is_internal() {
        return None;
    }

    // Find the unique item.
    let mut from = match data.left_table.get(&(state, token.symbol)) {
        Some(&item) => (state, item),
        None => return None,
    };

    let item = data.items[&from.1];
    let rule = grammar.rule(item.rule);
    let mut tail = rule.tail(item.dot).to_vec();
    let mut current_rule = rule.id;

    // Follow item transitions to find the remaining symbols.
    loop {
        let item = data.items[&from.1];
        let rule = grammar.rule(item.rule);

        if rule.id != current_rule {
            tail.extend(rule.tail(item.dot).iter().skip(1));
            current_rule = rule.id;
        }

        from = match data.backtrack_table.get(&from) {
            Some(&to) => to,
            None => break,
        };
    }

    Some((rule.id, tail))
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
fn reduce_rule(stack: &mut Vec<(usize, usize)>, symbols: &[usize]) -> Result<(), Error> {
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

#[derive(Debug)]
pub enum Error {
    Conflict(Symbol),
    EOF,
    Internal,
    Parse(Token),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Conflict(symbol) => write!(f, "Conflict in table for {}", symbol),
            Self::EOF => write!(f, "Unexpected end of file"),
            Self::Internal => write!(f, "Internal error"),
            Self::Parse(token) => write!(f, "Unexpected token {}", token),
        }
    }
}
impl error::Error for Error {}
