use crate::grammar::Grammar;
use crate::util::{self, AsString, Table};

use super::action::Action;

/// The `Data` struct contains all automaton data tables.
pub struct Data {
    pub action_table: Table<Action>,
    pub goto_table: Table<usize>,
    pub unique_table: Table<usize>,
    pub parse_table: Table<usize>,
}

impl AsString for Data {
    fn string(&self, grammar: &Grammar) -> String {
        let action_table = util::to_sorted_vec(&self.action_table)
            .iter()
            .map(|(&(state, symbol), action)| {
                format!("{}, {} → {}", state, grammar.symbol(symbol), action)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let goto_table = util::to_sorted_vec(&self.goto_table)
            .iter()
            .map(|(&(from, symbol), to)| format!("{}, {} → {}", from, grammar.symbol(symbol), to))
            .collect::<Vec<String>>()
            .join("\n");

        let unique_table = util::to_sorted_vec(&self.unique_table)
            .iter()
            .map(|(&(state, symbol), item)| {
                format!("{}, {} → {}", state, grammar.symbol(symbol), item)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let parse_table = util::to_sorted_vec(&self.parse_table)
            .iter()
            .map(|((from_item, state), to_item)| {
                format!("{}, {} → {}", from_item, state, to_item)
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            "ACTION\n{}\n\nGOTO\n{}\n\nUNIQUE\n{}\n\nPARSE\n{}",
            action_table, goto_table, unique_table, parse_table
        )
    }
}
