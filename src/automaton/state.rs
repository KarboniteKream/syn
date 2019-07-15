use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};

use indexmap::IndexSet;

use crate::grammar::Grammar;
use crate::rule::Rule;
use crate::symbol::Symbol;
use crate::util::{self, AsString};

use super::item::Item;
use super::transition::ItemTransition;
use super::Automaton;

#[derive(Clone, Debug)]
pub struct State {
    pub id: usize,
    pub items: Vec<usize>,
}

impl State {
    pub fn new(id: usize, items: Vec<usize>) -> State {
        State { id, items }
    }

    pub fn transitions(&self, items: &IndexSet<Item>) -> Vec<usize> {
        let transitions: HashSet<usize> = self
            .items
            .iter()
            .filter_map(|id| items.get_index(*id).unwrap().head)
            .filter(|head| *head != Symbol::Null.id())
            .collect();

        util::to_sorted_vec(transitions)
    }

    pub fn derive(
        &self,
        symbol: usize,
        grammar: &Grammar,
        items: &mut IndexSet<Item>,
        id: usize,
    ) -> Option<(State, HashSet<ItemTransition>)> {
        let mut next_items: Vec<Item> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, id)| {
                let mut item = items.get_index(*id).unwrap().clone();
                item.id = idx;
                item
            })
            .filter(|item| match item.head {
                Some(head) => head == symbol,
                None => false,
            })
            .collect();

        if next_items.is_empty() {
            return None;
        }

        let mut transitions: HashSet<ItemTransition> = HashSet::new();

        let mut buffer = HashSet::new();
        let mut queue = VecDeque::new();

        for (idx, item) in next_items.iter_mut().enumerate() {
            transitions.insert(ItemTransition::new((self.id, item.id), (id, idx), symbol));

            item.id = idx;
            item.pass(grammar.rule(item.rule));

            if item.at_nonterminal(&grammar.symbols) {
                queue.push_back(item.clone());
            }

            buffer.insert(item.clone());
            let mut item = item.clone();
            item.unique = !item.unique;
            buffer.insert(item);
        }

        while let Some(item) = queue.pop_front() {
            let head = item.head.unwrap();
            let tail = item.tail(grammar.rule(item.rule));
            let lookaheads = grammar.first_sequence(&tail);

            for rule in grammar.rules(head) {
                for lookahead in &lookaheads {
                    let rule = Rule::new(rule.id, head, rule.body.clone());
                    let mut next_item = Item::new(next_items.len(), &rule, *lookahead, item.unique);
                    let mut transition =
                        ItemTransition::new((id, item.id), (id, next_item.id), Symbol::Null.id());

                    if let Some(existing) = buffer.get(&next_item) {
                        transition.to.1 = existing.id;
                        transitions.insert(transition);

                        if !existing.unique {
                            continue;
                        }

                        let parents: Vec<&Item> = transitions
                            .iter()
                            .filter(|transition| transition.to.1 == existing.id)
                            .map(|transition| transition.from.1)
                            .filter_map(|idx| next_items.get(idx))
                            .collect();

                        next_items[existing.id].unique = parents.iter().all(|item| {
                            item.unique
                                && item.rule == parents[0].rule
                                && item.dot == parents[0].dot
                        });

                        continue;
                    }

                    if next_item.at_nonterminal(&grammar.symbols) {
                        queue.push_back(next_item.clone());
                    }

                    next_items.push(next_item.clone());
                    transitions.insert(transition);

                    buffer.insert(next_item.clone());
                    next_item.unique = !next_item.unique;
                    buffer.insert(next_item);
                }
            }
        }

        let next_items = next_items
            .iter()
            .map(|item| {
                if let Some(existing) = items.get(item) {
                    return existing.id;
                }

                let id = items.len();
                let mut item = item.clone();
                item.id = id;
                items.insert(item);
                id
            })
            .collect();

        Some((State::new(id, next_items), transitions))
    }

    pub fn string(&self, automaton: &Automaton) -> String {
        let items = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, id)| {
                let mut item = automaton.items[*id].clone();
                item.id = idx;
                item.string(&automaton.grammar)
            })
            .collect::<Vec<String>>()
            .join("; ");

        format!("({}) [{}]", self.id, items)
    }
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.items == other.items
    }
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}
