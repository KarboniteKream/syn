use std::collections::{HashSet, VecDeque};
use std::fmt::{self, Display, Formatter};

use crate::grammar::Grammar;
use crate::symbol::Symbol;
use crate::util;

use super::state::State;
use super::transition::{ItemTransition, StateTransition};

pub struct Automaton {
    grammar: Grammar,
    states: Vec<State>,
    state_transitions: Vec<StateTransition>,
    item_transitions: Vec<ItemTransition>,
}

impl Automaton {
    pub fn new(grammar: &Grammar) -> Automaton {
        let grammar = grammar.clone();

        let mut states = Vec::new();
        let mut buffer = HashSet::new();
        let mut queue = VecDeque::new();

        let mut state_transitions = HashSet::new();
        let mut item_transitions = HashSet::new();

        let initial_state = State::initial(&grammar);
        states.push(initial_state.clone());
        buffer.insert(initial_state.clone());
        enqueue(&mut queue, &initial_state);

        while let Some((state, symbol)) = queue.pop_front() {
            let (mut next_state, transitions) =
                state.derive(&grammar, &symbol, states.len()).unwrap();
            let mut state_transition = StateTransition::new(state.id, next_state.id, symbol);

            if let Some(state) = buffer.get(&next_state) {
                state_transition.to = state.id;
                state_transitions.insert(state_transition);

                for transition in transitions {
                    let mut transition = transition.clone();

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
            states.push(next_state.clone());
            buffer.insert(next_state.clone());
            state_transitions.insert(state_transition);
            item_transitions.extend(transitions);
            enqueue(&mut queue, &next_state);
        }

        Automaton {
            grammar,
            states,
            state_transitions: util::to_sorted_vec(&state_transitions),
            item_transitions: util::to_sorted_vec(&item_transitions),
        }
    }

    pub fn to_dot(&self) -> String {
        let nodes = self
            .states
            .iter()
            .map(|state| {
                let items = state
                    .items
                    .iter()
                    .map(|item| {
                        let label = item.to_string().replace("\"", "\\\"");
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

        let state_edges = self
            .state_transitions
            .iter()
            .map(|StateTransition { from, to, symbol }| {
                let label = symbol.to_string().replace("\"", "\\\"");
                format!("    {} -> {} [label=\"{}\"];", from, to, label)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let item_edges = self
            .item_transitions
            .iter()
            .map(|ItemTransition { from, to, symbol }| {
                let color = match symbol {
                    Symbol::Null => {
                        if from.1 < to.1 {
                            "crimson"
                        } else {
                            "forestgreen"
                        }
                    }
                    _ => "royalblue",
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
            self.grammar.name, nodes, state_edges, item_edges
        )
    }
}

fn enqueue(queue: &mut VecDeque<(State, Symbol)>, state: &State) {
    for transition in state.transitions() {
        queue.push_back((state.clone(), transition.clone()));
    }
}

impl Display for Automaton {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let states = util::to_string(self.states.iter(), "\n");
        let state_transitions = util::to_string(self.state_transitions.iter(), "\n");
        let item_transitions = util::to_string(self.item_transitions.iter(), "\n");

        write!(
            f,
            "{}\n\n{}\n\n{}",
            states, state_transitions, item_transitions
        )
    }
}
