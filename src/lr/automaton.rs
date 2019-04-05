use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};

use super::state::State;
use crate::grammar::Grammar;
use crate::symbol::Symbol;

pub struct Automaton {
    states: Vec<State>,
}

impl Automaton {
    pub fn new(grammar: &Grammar) -> Automaton {
        let mut states = Vec::new();
        let mut queue: VecDeque<(State, Symbol)> = VecDeque::new();

        let initial_state = State::initial(grammar);
        states.push(initial_state.clone());
        enqueue(&mut queue, &initial_state);

        while let Some((state, symbol)) = queue.pop_front() {
            let next_state = state.derive(&grammar, &symbol).unwrap();

            if !states.contains(&next_state) {
                states.push(next_state.clone());
                enqueue(&mut queue, &next_state);
            }
        }

        Automaton { states }
    }
}

fn enqueue(queue: &mut VecDeque<(State, Symbol)>, state: &State) {
    for transition in state.transitions() {
        queue.push_back((state.clone(), transition.clone()));
    }
}

impl Display for Automaton {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let states = self
            .states
            .iter()
            .map(State::to_string)
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{}", states)
    }
}
