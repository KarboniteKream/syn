use std::collections::HashSet;

use crate::grammar::Grammar;
use crate::rule::Rule;
use crate::symbol::Symbol;

mod item;
mod state;

use self::item::Item;
use self::state::State;

pub fn initial_state(grammar: &Grammar) -> State {
    let rule = Rule::new(
        Symbol::NonTerminal(grammar.start_symbol.name().to_owned() + "'"),
        vec![
            Symbol::Delimiter,
            grammar.start_symbol.clone(),
            Symbol::Delimiter,
        ],
    );

    let item = Item::new(rule, Symbol::Null);
    State::new(vec![item])
}

pub fn next_state(state: &State, grammar: &Grammar, path: &Symbol) -> Option<State> {
    let items: Vec<Item> = state
        .items
        .iter()
        .filter(|item| match item.head() {
            Some(symbol) => symbol == path,
            None => false,
        })
        .cloned()
        .collect();

    if items.is_empty() {
        return None;
    }

    let mut next_state: State = State::new(items);
    for item in &mut next_state.items {
        item.consume();
    }

    let new_items: Vec<Item> = next_state
        .items
        .iter()
        .filter(|item| match item.head() {
            Some(symbol) => symbol.is_nonterminal(),
            None => false,
        })
        .flat_map(|item| {
            let head: &Symbol = item.head().unwrap();
            let tail_first: HashSet<Symbol> = grammar.first_sequence(&item.tail());

            grammar.rules[head]
                .iter()
                .flat_map(|rule| {
                    tail_first
                        .iter()
                        .map(|first| {
                            let rule = Rule::new(head.clone(), rule.body.clone());
                            Item::new(rule, first.clone())
                        })
                        .collect::<Vec<Item>>()
                })
                .collect::<Vec<Item>>()
        })
        .collect();

    next_state.items.extend(new_items);
    Some(next_state)
}
