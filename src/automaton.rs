use std::collections::{HashMap, HashSet, VecDeque};
use std::error;
use std::fmt::{self, Display, Formatter};

mod action;
mod item;
mod state;
mod transition;

use crate::grammar::Grammar;
use crate::symbol::Symbol;
use crate::util::{self, AsString};

use action::Action;
use item::Item;
use state::State;
use transition::{ItemTransition, StateTransition};

pub struct Automaton {
    pub grammar: Grammar,
    states: Vec<State>,
    state_transitions: Vec<StateTransition>,
    items: Vec<Item>,
    item_transitions: Vec<ItemTransition>,
}

impl Automaton {
    pub fn new(grammar: Grammar) -> Automaton {
        let mut states = HashSet::new();
        let mut state_transitions = HashSet::new();
        let mut items = HashSet::new();
        let mut item_transitions = HashSet::new();
        let mut queue = VecDeque::new();

        let initial_state = State::initial(&grammar);
        states.insert(initial_state.clone());
        items.insert(initial_state.items[0].clone());
        enqueue(&mut queue, &initial_state);

        while let Some((state, symbol)) = queue.pop_front() {
            let (mut next_state, transitions) =
                state.derive(symbol, &grammar, states.len()).unwrap();
            let mut state_transition = StateTransition::new(state.id, next_state.id, symbol);

            if let Some(state) = states.get(&next_state) {
                state_transition.to = state.id;
                state_transitions.insert(state_transition);

                for mut transition in transitions {
                    if transition.from.0 == next_state.id {
                        transition.from.0 = state.id;
                    }

                    if transition.to.0 == next_state.id {
                        transition.to.0 = state.id;
                    }

                    item_transitions.insert(transition);
                }

                continue;
            }

            next_state.id = states.len();
            states.insert(next_state.clone());
            state_transitions.insert(state_transition);

            for item in &next_state.items {
                if !items.contains(&item) {
                    let mut item = item.clone();
                    item.id = items.len();
                    items.insert(item);
                }
            }

            item_transitions.extend(transitions);
            enqueue(&mut queue, &next_state);
        }

        Automaton {
            grammar,
            states: util::to_sorted_vec(&states),
            state_transitions: util::to_sorted_vec(&state_transitions),
            items: util::to_sorted_vec(&items),
            item_transitions: util::to_sorted_vec(&item_transitions),
        }
    }

    pub fn item(&self, state: usize, item: usize) -> &Item {
        &self.states[state].items[item]
    }

    pub fn action_table(&self) -> Result<HashMap<(usize, usize), Action>, Error> {
        let mut action_table: HashMap<(usize, usize), Action> = self
            .state_transitions
            .iter()
            .filter_map(|transition| match self.grammar.symbol(transition.symbol) {
                Symbol::Terminal(..) => Some(*transition),
                _ => None,
            })
            .map(|StateTransition { from, to, symbol }| ((from, symbol), Action::Shift(to)))
            .collect();

        for state in &self.states {
            for item in &state.items {
                if item.can_accept() {
                    action_table.insert((state.id, Symbol::End.id()), Action::Accept);
                    continue;
                }

                if !item.can_reduce() {
                    continue;
                }

                let key = (state.id, item.lookahead);

                if action_table.contains_key(&key) {
                    let symbol = self.grammar.symbol(key.1).clone();
                    return Err(Error::ActionConflict(key.0, symbol));
                }

                action_table.insert(key, Action::Reduce(item.rule));
            }
        }

        Ok(action_table)
    }

    pub fn goto_table(&self) -> HashMap<(usize, usize), usize> {
        self.state_transitions
            .iter()
            .filter_map(|transition| match self.grammar.symbol(transition.symbol) {
                Symbol::NonTerminal(..) => Some(*transition),
                _ => None,
            })
            .map(|StateTransition { from, to, symbol }| ((from, symbol), to))
            .collect()
    }

    pub fn unique_table(&self) -> HashMap<(usize, usize), usize> {
        self.states
            .iter()
            .flat_map(|state| {
                state.items.iter().fold(HashMap::new(), |mut acc, item| {
                    let follow = item.follow(self.grammar.rule(item.rule));

                    for symbol in self.grammar.first_sequence(&follow) {
                        acc.entry((state.id, symbol))
                            .or_insert_with(Vec::new)
                            .push(item);
                    }

                    acc
                })
            })
            .filter(|(_, items)| items.len() == 1 && items[0].unique)
            .map(|(key, items)| (key, items[0].id))
            .collect()
    }

    pub fn parse_table(&self) -> HashMap<(usize, usize), usize> {
        let items: HashSet<&Item> = self.items.iter().collect();

        self.item_transitions
            .iter()
            .map(|ItemTransition { from, to, .. }| {
                let state = to.0;

                let from = items.get(self.item(from.0, from.1)).unwrap().id;
                let to = items.get(self.item(to.0, to.1)).unwrap().id;

                ((to, state), from)
            })
            .collect()
    }

    pub fn to_dot(&self) -> String {
        let states = self
            .states
            .iter()
            .map(|state| {
                let items = state
                    .items
                    .iter()
                    .map(|item| {
                        let label = item.string(&self.grammar).replace("\"", "\\\"");
                        format!("<{}> {}", item.id, label)
                    })
                    .collect::<Vec<String>>()
                    .join(" | ");

                format!(
                    "    {0} [label=\"{0} | {1}\", shape=record];",
                    state.id, items
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let state_transitions = self
            .state_transitions
            .iter()
            .map(|StateTransition { from, to, symbol }| {
                let label = self
                    .grammar
                    .symbol(*symbol)
                    .to_string()
                    .replace("\"", "\\\"");
                format!("    {} -> {} [label=\"{}\"];", from, to, label)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let item_transitions = self
            .item_transitions
            .iter()
            .map(|ItemTransition { from, to, symbol }| {
                let color = if *symbol == Symbol::Null.id() {
                    if from.1 < to.1 {
                        "crimson"
                    } else {
                        "forestgreen"
                    }
                } else {
                    "royalblue"
                };

                format!(
                    "    {}:{} -> {}:{} [color={}];",
                    from.0, from.1, to.0, to.1, color
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            "digraph {0} {{\n    label=\"{0}\";\n    rankdir=LR;\n\n{1}\n\n{2}\n\n{3}\n}}",
            self.grammar.name, states, state_transitions, item_transitions
        )
    }
}

fn enqueue(queue: &mut VecDeque<(State, usize)>, state: &State) {
    for symbol in state.transitions() {
        queue.push_back((state.clone(), symbol));
    }
}

impl Display for Automaton {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let states = format!(
            "States\n{}\n\nState transitions\n{}",
            util::as_string(self.states.iter(), &self.grammar, "\n"),
            util::as_string(self.state_transitions.iter(), &self.grammar, "\n")
        );

        let items = format!(
            "Items\n{}\n\nItem transitions\n{}",
            util::as_string(self.items.iter(), &self.grammar, "\n"),
            util::as_string(self.item_transitions.iter(), &self.grammar, "\n")
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
            Error::ActionConflict(state, symbol) => {
                write!(f, "Shift/Reduce conflict in ACTION({}, {})", state, symbol)
            }
        }
    }
}

impl error::Error for Error {}
