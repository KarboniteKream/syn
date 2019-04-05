use std::cmp;
use std::collections::{HashSet, VecDeque};
use std::fmt::{self, Display, Formatter};

use super::state::State;
use crate::grammar::Grammar;
use crate::symbol::Symbol;

type Transition = (usize, usize, Symbol);

pub struct Automaton {
    grammar: Grammar,
    states: Vec<State>,
    transitions: Vec<Transition>,
}

impl Automaton {
    pub fn new(grammar: &Grammar) -> Automaton {
        let grammar = grammar.clone();
        let mut states = Vec::new();
        let mut transitions = HashSet::new();

        let mut queue: VecDeque<(State, Symbol)> = VecDeque::new();

        let initial_state = State::initial(&grammar);
        states.push(initial_state.clone());
        enqueue(&mut queue, &initial_state);

        while let Some((state, symbol)) = queue.pop_front() {
            let mut next_state = state.derive(&grammar, &symbol).unwrap();

            if let Some(existing) = states.iter().find(|state| **state == next_state) {
                transitions.insert((state.id, existing.id, symbol));
            } else {
                next_state.id = states.len();
                states.push(next_state.clone());
                transitions.insert((state.id, next_state.id, symbol));
                enqueue(&mut queue, &next_state);
            }
        }

        let mut transitions: Vec<Transition> = transitions.into_iter().collect();
        transitions.sort_unstable();

        Automaton {
            grammar,
            states,
            transitions,
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
                    .map(|item| item.to_string().replace("\"", "\\\""))
                    .collect::<Vec<String>>()
                    .join("\\n");

                format!(
                    "    {0} [label=\"{0}|{1}\", shape=record];",
                    state.id, items
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let edges = self
            .transitions
            .iter()
            .map(|(left, right, symbol)| {
                format!(
                    "    {} -> {} [label=\"{}\"];",
                    left,
                    right,
                    symbol.to_string().replace("\"", "\\\"")
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            "digraph {0} {{\n    label=\"{0}\";\n    rankdir=LR;\n\n{1}\n\n{2}\n}}",
            self.grammar.name, nodes, edges
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
        let (left, right) = self
            .transitions
            .iter()
            .map(|(from, to, _)| (from.to_string().len(), to.to_string().len()))
            .fold((0, 0), |(left, right), (from, to)| {
                (cmp::max(left, from), cmp::max(right, to))
            });

        let states = self
            .states
            .iter()
            .map(|state| {
                format!(
                    "{:<width$} {}",
                    state.id,
                    state,
                    width = cmp::max(left, right)
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let transitions = self
            .transitions
            .iter()
            .map(|(from, to, symbol)| {
                format!(
                    "{:<left$} → {:<right$} {}",
                    from,
                    to,
                    symbol,
                    left = left,
                    right = right,
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{}\n\n{}", states, transitions)
    }
}
