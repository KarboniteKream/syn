use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};

use indexmap::IndexSet;

use crate::grammar::{Grammar, Symbol};
use crate::util::{self, AsString};

use super::item::Item;
use super::transition::ItemTransition;
use super::Automaton;

/// The `State` struct describes a state in the automaton.
///
/// To ensure the relation between `Eq` and `Ord`, the fields
/// in the struct _must_ be unique for a specific `id`.
#[derive(Clone, Debug, Eq)]
pub struct State {
    pub id: usize,
    pub items: Vec<usize>,
}

impl State {
    /// Constructs a new automaton state.
    pub fn new(id: usize, items: Vec<usize>) -> State {
        State { id, items }
    }

    /// Returns the list of transitions from the state.
    pub fn transitions(&self, items: &IndexSet<Item>) -> Vec<usize> {
        let transitions: HashSet<usize> = self
            .items
            .iter()
            .filter_map(|&id| items.get_index(id).unwrap().head)
            .filter(|&head| head != Symbol::Null.id())
            .collect();

        util::to_sorted_vec(transitions)
    }

    /// Derives the next state with specified transition symbol.
    pub fn derive(
        &self,
        symbol: usize,
        grammar: &Grammar,
        items: &mut IndexSet<Item>,
        state_id: usize,
    ) -> Option<(State, HashSet<ItemTransition>)> {
        // Keep all items which can transition with the symbol
        // and assign them a temporary index-based ID.
        let mut next_items: Vec<Item> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, &id)| {
                let mut item = *items.get_index(id).unwrap();
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

        // The buffer contains unique and non-unique versions
        // of items so we can find existing items easier.
        let mut buffer = HashSet::new();
        let mut queue = VecDeque::new();

        // Generate initial transitions and derive the items.
        // Assign them a new temporary index-based ID.
        for (idx, item) in next_items.iter_mut().enumerate() {
            transitions.insert(ItemTransition::new(
                (self.id, item.id),
                (state_id, idx),
                symbol,
            ));

            item.id = idx;
            item.pass(grammar.rule(item.rule));

            if item.at_nonterminal(&grammar.symbols) {
                queue.push_back(item.id);
            }

            let mut item = *item;
            buffer.insert(item);
            item.unique = !item.unique;
            buffer.insert(item);
        }

        while let Some(item) = queue.pop_front() {
            let item = next_items[item];

            let head = item.head.unwrap();
            let tail = item.tail(grammar.rule(item.rule));
            // Find the FIRST set of the symbol sequence
            // that follows the current item's head.
            let lookaheads = grammar.first_sequence(&tail);

            // Find all the grammar rules for the current item head.
            for rule in grammar.rules(head) {
                // Derive a new item for all the symbols in the FIRST set.
                for &lookahead in &lookaheads {
                    let mut next_item = Item::new(next_items.len(), &rule, lookahead, item.unique);
                    let mut transition = ItemTransition::new(
                        (state_id, item.id),
                        (state_id, next_item.id),
                        Symbol::Null.id(),
                    );

                    // If the item derives an existing one, update it accordingly.
                    if let Some(existing) = buffer.get(&next_item) {
                        let existing = next_items[existing.id];

                        transition.to.1 = existing.id;
                        transitions.insert(transition);

                        update_uniqueness(
                            existing.id,
                            (state_id, item.id),
                            &mut next_items,
                            &transitions,
                        );

                        continue;
                    }

                    // If the item is at a nonterminal symbol, add it to the queue.
                    if next_item.at_nonterminal(&grammar.symbols) {
                        queue.push_back(next_item.id);
                    }

                    next_items.push(next_item);
                    transitions.insert(transition);

                    // Add both variations to the buffer.
                    buffer.insert(next_item);
                    next_item.unique = !next_item.unique;
                    buffer.insert(next_item);
                }
            }
        }

        // Update items with final unique IDs.
        let mut next_items: Vec<usize> = next_items
            .iter()
            .map(|item| {
                if let Some(existing) = items.get(item) {
                    return existing.id;
                }

                let id = items.len();
                let mut item = *item;
                item.id = id;
                items.insert(item);
                id
            })
            .collect();

        // Update transitions with correct item IDs.
        let transitions = transitions
            .into_iter()
            .map(|ItemTransition { from, to, symbol }| {
                let from = if from.0 == self.id {
                    (from.0, self.items[from.1])
                } else {
                    (from.0, next_items[from.1])
                };

                let to = if to.0 == self.id {
                    (to.0, self.items[to.1])
                } else {
                    (to.0, next_items[to.1])
                };

                ItemTransition::new(from, to, symbol)
            })
            .collect();

        next_items.sort_unstable();
        Some((State::new(state_id, next_items), transitions))
    }

    /// Different implementation of the `AsString` trait.
    pub fn string(&self, automaton: &Automaton) -> String {
        let items = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, &id)| {
                let mut item = automaton.items[id];
                item.id = idx;
                item.string(&automaton.grammar)
            })
            .collect::<Vec<String>>()
            .join("; ");

        format!("({}) [{}]", self.id, items)
    }
}

/// Updates the item uniqueness if necessary.
fn update_uniqueness(
    id: usize,
    from: (usize, usize),
    items: &mut Vec<Item>,
    transitions: &HashSet<ItemTransition>,
) {
    if !items[id].unique {
        return;
    }

    if id == from.1 {
        // If the item derives itself, it's not unique.
        items[id].unique = false;
    } else {
        // All its parents must be unique and represent the same rule.
        let parents: Vec<&Item> = transitions
            .iter()
            .filter(|transition| transition.to.1 == id)
            .map(|transition| transition.from.1)
            .filter_map(|idx| items.get(idx))
            .collect();

        items[id].unique = parents
            .iter()
            .all(|item| item.unique && item.rule == parents[0].rule && item.dot == parents[0].dot);
    }

    if items[id].unique {
        return;
    }

    // Recursively update its derived items.
    let mut non_unique = HashSet::new();
    non_unique.insert(id);

    loop {
        non_unique = transitions
            .iter()
            .filter(|transition| {
                transition.from.0 == from.0
                    && items[transition.to.1].unique
                    && non_unique.contains(&transition.from.1)
            })
            .map(|transition| transition.to.1)
            .collect();

        if non_unique.is_empty() {
            break;
        }

        for &id in &non_unique {
            items[id].unique = false;
        }
    }
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.items == other.items
    }
}

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
