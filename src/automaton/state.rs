use std::collections::{HashSet, VecDeque};
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::grammar::Grammar;
use crate::rule::Rule;
use crate::symbol::Symbol;
use crate::util;

use super::item::Item;
use super::transition::ItemTransition;

#[derive(Clone, Debug)]
pub struct State {
    pub id: usize,
    pub items: Vec<Item>,
}

impl State {
    pub fn new(id: usize, items: Vec<Item>) -> State {
        State { id, items }
    }

    pub fn initial(grammar: &Grammar) -> State {
        let rule = Rule::new(
            0,
            Symbol::NonTerminal(grammar.start_symbol.name().to_owned() + "'"),
            vec![
                Symbol::Delimiter,
                grammar.start_symbol.clone(),
                Symbol::Delimiter,
            ],
        );

        State::new(0, vec![Item::new(0, rule, Symbol::Null, true)])
    }

    pub fn transitions(&self) -> Vec<&Symbol> {
        let transitions: HashSet<&Symbol> = self
            .items
            .iter()
            .filter_map(Item::head)
            .filter(|symbol| **symbol != Symbol::Null)
            .collect();

        util::to_sorted_vec(&transitions)
    }

    pub fn derive(
        &self,
        grammar: &Grammar,
        symbol: &Symbol,
        id: usize,
    ) -> Option<(State, HashSet<ItemTransition>)> {
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

        let mut transitions: HashSet<ItemTransition> = HashSet::new();

        let mut buffer = HashSet::new();
        let mut queue = VecDeque::new();

        for (idx, item) in items.iter_mut().enumerate() {
            transitions.insert(ItemTransition::new(
                (self.id, item.id),
                (id, idx),
                symbol.clone(),
            ));

            item.id = idx;
            item.pass();

            if item.at_nonterminal() {
                queue.push_back(item.clone());
            }

            buffer.insert(item.clone());
            let mut item = item.clone();
            item.unique = !item.unique;
            buffer.insert(item);
        }

        while let Some(item) = queue.pop_front() {
            let head: &Symbol = item.head().unwrap();
            let lookaheads: Vec<Symbol> = grammar.first_sequence(&item.tail());

            for rule in &grammar.rules[head] {
                for lookahead in lookaheads.clone() {
                    let rule = Rule::new(rule.id, head.clone(), rule.body.clone());
                    let mut next_item = Item::new(items.len(), rule, lookahead, item.unique);

                    let mut transition =
                        ItemTransition::new((id, item.id), (id, next_item.id), Symbol::Null);

                    if let Some(item) = buffer.get(&next_item) {
                        transition.to.1 = item.id;
                        transitions.insert(transition);

                        let parents: Vec<&Item> = transitions
                            .iter()
                            .filter(|transition| transition.to.1 == item.id)
                            .map(|transition| transition.from.1)
                            .filter_map(|item_id| {
                                items.get(item_id).or_else(|| self.items.get(item_id))
                            })
                            .collect();

                        items[item.id].unique = parents.iter().all(|item| {
                            item.unique
                                && item.rule == parents[0].rule
                                && item.idx == parents[0].idx
                        });

                        continue;
                    }

                    if next_item.at_nonterminal() {
                        queue.push_back(next_item.clone());
                    }

                    items.push(next_item.clone());
                    transitions.insert(transition);

                    buffer.insert(next_item.clone());
                    next_item.unique = !next_item.unique;
                    buffer.insert(next_item);
                }
            }
        }

        Some((State::new(id, items), transitions))
    }
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.items == other.items
    }
}

impl Eq for State {}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let items = util::to_string(self.items.iter(), "; ");
        write!(f, "{} [{}]", self.id, items)
    }
}