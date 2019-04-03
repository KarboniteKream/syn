use std::collections::{HashSet, VecDeque};
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

    pub fn transitions(&self) -> Vec<&Symbol> {
        self.items
            .iter()
            .filter_map(Item::head)
            .filter(|symbol| **symbol != Symbol::Null)
            .collect()
    }

    pub fn derive(&self, grammar: &Grammar, symbol: &Symbol) -> Option<State> {
        let mut items: Vec<Item> = self
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

        let mut buffer: HashSet<Item> = items.iter().cloned().collect();
        let mut queue = VecDeque::new();

        for item in &mut items {
            item.pass();

            if item.is_nonterminal() {
                queue.push_back(item.clone());
            }
        }

        while let Some(item) = queue.pop_front() {
            let head: &Symbol = item.head().unwrap();
            let first: Vec<Symbol> = grammar.first_sequence(&item.tail());

            for rule in &grammar.rules[head] {
                for sym in &first {
                    let rule = Rule::new(head.clone(), rule.body.clone());
                    let item = Item::new(rule, sym.clone());

                    if buffer.contains(&item) {
                        continue;
                    }

                    if item.is_nonterminal() {
                        queue.push_back(item.clone());
                    }

                    buffer.insert(item.clone());
                    items.push(item);
                }
            }
        }

        Some(State::new(items))
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
