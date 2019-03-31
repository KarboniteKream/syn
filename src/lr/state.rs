use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};

use super::item::Item;
use crate::grammar::Grammar;
use crate::rule::Rule;
use crate::symbol::Symbol;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    pub items: Vec<Item>,
}

impl State {
    pub fn new(items: Vec<Item>) -> State {
        State { items }
    }

    pub fn initial(grammar: &Grammar) -> State {
        let rule = Rule::new(
            Symbol::NonTerminal(grammar.start_symbol.name().to_owned() + "'"),
            vec![
                Symbol::Delimiter,
                grammar.start_symbol.clone(),
                Symbol::Delimiter,
            ],
        );

        State::new(vec![Item::new(rule, Symbol::Null)])
    }

    pub fn transitions(&self) -> HashSet<&Symbol> {
        self.items
            .iter()
            .filter_map(Item::head)
            .filter(|symbol| **symbol != Symbol::Null)
            .collect()
    }

    pub fn derive(&self, grammar: &Grammar, symbol: &Symbol) -> Option<State> {
        let items: Vec<Item> = self
            .items
            .iter()
            .filter(|item| match item.head() {
                Some(head) => head == symbol,
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
                Some(head) => head.is_nonterminal(),
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
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let items = self
            .items
            .iter()
            .map(Item::to_string)
            .collect::<Vec<String>>()
            .join(";  ");

        write!(f, "[{}]", items)
    }
}
