use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error;
use std::fmt::{self, Display, Formatter};

use indexmap::IndexSet;

use crate::grammar::{Grammar, Symbol};
use crate::util::{self, AsString};

mod action;
mod data;
mod item;
mod state;
mod transition;

pub use action::Action;
pub use data::{Data, Table};
pub use item::Item;
use state::State;
use transition::{ItemTransition, StateTransition};

/// The `Automaton` struct describes the automaton for a grammar.
pub struct Automaton {
    grammar: Grammar,
    start_rule: usize,
    states: Vec<State>,
    state_transitions: Vec<StateTransition>,
    items: Vec<Item>,
    item_transitions: Vec<ItemTransition>,
}

impl Automaton {
    /// Constructs a new automaton.
    pub fn new(grammar: &Grammar, rule: usize) -> Automaton {
        let mut queue = VecDeque::new();

        let mut states = IndexSet::new();
        let mut state_transitions = HashSet::new();
        let mut items = IndexSet::new();
        let mut item_transitions = HashSet::new();

        // Construct the initial state.
        let start_rule = grammar.rule(rule);

        for &lookahead in &start_rule.follow {
            items.insert(Item::initial(items.len(), start_rule, lookahead));
        }

        let initial_state = State::new(0, (0..items.len()).collect());
        queue.push_back((initial_state.id, Symbol::End.id()));
        states.insert(initial_state);

        while let Some((id, symbol)) = queue.pop_front() {
            let state = states.get_index(id).unwrap();

            // Derive the next state from the current state
            // using the specified transition symbol.
            let (mut next_state, transitions) = state
                .derive(symbol, grammar, &mut items, states.len())
                .unwrap();

            let mut state_transition = StateTransition::new(state.id, next_state.id, symbol);

            // If the derived state already exists, save the
            // derived transitions using the existing ID.
            if let Some(existing) = states.get(&next_state) {
                state_transition.to = existing.id;
                state_transitions.insert(state_transition);

                for mut transition in transitions {
                    if transition.from.0 == next_state.id {
                        transition.from.0 = existing.id;
                    }

                    if transition.to.0 == next_state.id {
                        transition.to.0 = existing.id;
                    }

                    item_transitions.insert(transition);
                }

                continue;
            }

            next_state.id = states.len();
            states.insert(next_state.clone());

            state_transitions.insert(state_transition);
            item_transitions.extend(transitions);

            // Add the state with its transition symbols to the queue.
            for symbol in next_state.transitions(&items) {
                queue.push_back((next_state.id, symbol));
            }
        }

        Automaton {
            grammar: grammar.clone(),
            start_rule: start_rule.id,
            states: util::to_sorted_vec(states),
            state_transitions: util::to_sorted_vec(state_transitions),
            items: util::to_sorted_vec(items),
            item_transitions: util::to_sorted_vec(item_transitions),
        }
    }

    /// Returns all automaton data tables.
    pub fn data(&self) -> Result<Data, Error> {
        let action_table = self.action_table()?;

        Ok(Data::new(
            &self.grammar,
            self.start_rule,
            &self.items,
            action_table,
            self.goto_table(),
            self.left_table(),
            self.backtrack_table(),
        ))
    }

    /// Converts the automaton to the DOT format.
    pub fn to_dot(&self) -> String {
        // The list of states in the `record` format.
        // Each port of the node corresponds to an automaton item.
        let states = self
            .states
            .iter()
            .map(|state| {
                let items = state
                    .items
                    .iter()
                    .enumerate()
                    .map(|(idx, &id)| {
                        let mut item = self.items[id];
                        item.id = idx;

                        let label = item.string(&self.grammar).replace("\"", "\\\"");
                        format!("<{}> {}", item.id, label)
                    })
                    .collect::<Vec<String>>()
                    .join(" | ");

                format!("    {0} [label=\"{0} | {1}\"];", state.id, items)
            })
            .collect::<Vec<String>>()
            .join("\n");

        // Edges between nodes.
        let state_transitions = self
            .state_transitions
            .iter()
            .map(|&StateTransition { from, to, symbol }| {
                let label = self
                    .grammar
                    .symbol(symbol)
                    .to_string()
                    .replace("\"", "\\\"");
                format!("    {} -> {} [label=\"{}\"];", from, to, label)
            })
            .collect::<Vec<String>>()
            .join("\n");

        // Edges between node ports.
        let item_transitions = self
            .item_transitions
            .iter()
            .map(|&ItemTransition { from, to, symbol }| {
                let from_item = util::get_index(&self.states[from.0].items, from.1);
                let to_item = util::get_index(&self.states[to.0].items, to.1);

                let color = if symbol == Symbol::Null.id() {
                    if from_item < to_item {
                        "crimson"
                    } else {
                        "forestgreen"
                    }
                } else {
                    "royalblue"
                };

                format!(
                    "    {}:{} -> {}:{} [color={}];",
                    from.0, from_item, to.0, to_item, color
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let attributes = format!(
            "    label=\"{}\";\n    rankdir=LR;\n\n    node [shape=record];",
            self.grammar.name
        );

        format!(
            "digraph {} {{\n{}\n\n{}\n\n{}\n\n{}\n}}",
            self.grammar.name, attributes, states, state_transitions, item_transitions
        )
    }

    /// Returns the ACTION table of the automaton.
    fn action_table(&self) -> Result<Table<Action>, Error> {
        // All transitions with terminal symbols correspond to a Shift action.
        let mut action_table: Table<Action> = self
            .state_transitions
            .iter()
            .filter_map(|&transition| match self.grammar.symbol(transition.symbol) {
                Symbol::Terminal(..) | Symbol::End => Some(transition),
                _ => None,
            })
            .map(|StateTransition { from, to, symbol }| ((from, symbol), Action::Shift(to)))
            .collect();

        for state in &self.states {
            for &id in &state.items {
                let item = self.items[id];

                // Accept actions have a higher precedence.
                let (symbol, action) = if item.can_accept(self.start_rule) {
                    let symbol = item.head.unwrap_or(item.lookahead);
                    (symbol, Action::Accept(item.rule))
                } else if item.can_reduce(self.start_rule) {
                    (item.lookahead, Action::Reduce(item.rule))
                } else {
                    continue;
                };

                let key = (state.id, symbol);

                // Every symbol in each state can only correspond to a single action.
                if let Entry::Occupied(mut entry) = action_table.entry(key) {
                    // Resolve conflicts if the grammar has specified an override.
                    if let Some(preference) = self.grammar.actions.get(&symbol) {
                        if preference.is_reduce() {
                            entry.insert(action);
                        }

                        continue;
                    }

                    let symbol = self.grammar.symbol(symbol).clone();
                    return Err(Error::ActionConflict(state.id, symbol));
                }

                action_table.insert(key, action);
            }
        }

        Ok(action_table)
    }

    /// Returns the GOTO table of the automaton.
    fn goto_table(&self) -> Table<usize> {
        self.state_transitions
            .iter()
            .filter_map(|&transition| match self.grammar.symbol(transition.symbol) {
                Symbol::NonTerminal(..) => Some(transition),
                _ => None,
            })
            .map(|StateTransition { from, to, symbol }| ((from, symbol), to))
            .collect()
    }

    /// Returns the LEFT table of the automaton.
    ///
    /// The LEFT table describes which item a symbol in a state corresponds to.
    /// A symbol corresponds to an item, if it's in its FIRST set.
    fn left_table(&self) -> Table<usize> {
        self.states
            .iter()
            .flat_map(|state| {
                state.items.iter().fold(HashMap::new(), |mut acc, &id| {
                    let item = self.items[id];

                    // Find the FIRST set of the item's follow sequence.
                    let sequence = item.follow(self.grammar.rule(item.rule));
                    let first = self.grammar.first_sequence(&sequence);

                    for symbol in first {
                        let items: &mut Vec<Item> = acc.entry((state.id, symbol)).or_default();
                        items.push(item);
                    }

                    acc
                })
            })
            .filter(|(_, items)| {
                // The LEFT table only contains symbols
                // that correspond to a single, unique item.
                items.len() == 1 && items[0].unique
            })
            .map(|(key, items)| (key, items[0].id))
            .collect()
    }

    /// Returns the BACKTRACK table of the automaton.
    /// It describes reverse transitions between unique items.
    fn backtrack_table(&self) -> Table<(usize, usize)> {
        self.item_transitions
            .iter()
            .rev()
            .filter(|transition| self.items[transition.to.1].unique)
            .map(|transition| (transition.to, transition.from))
            .collect()
    }
}

impl Display for Automaton {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let states = format!(
            "States\n{}\n\nState transitions\n{}",
            self.states
                .iter()
                .map(|item| item.string(self))
                .collect::<Vec<String>>()
                .join("\n"),
            util::as_string(&self.state_transitions, &self.grammar, "\n")
        );

        let items = format!(
            "Items\n{}\n\nItem transitions\n{}",
            util::as_string(&self.items, &self.grammar, "\n"),
            util::as_string(&self.item_transitions, &self.grammar, "\n")
        );

        write!(f, "{}\n\n{}", states, items)
    }
}

#[derive(Debug)]
pub enum Error {
    ActionConflict(usize, Symbol),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ActionConflict(state, symbol) => {
                write!(f, "Shift/Reduce conflict in ACTION({}, {})", state, symbol)
            }
        }
    }
}

impl error::Error for Error {}
