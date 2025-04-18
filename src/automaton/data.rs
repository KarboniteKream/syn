use std::collections::HashMap;

use crate::grammar::Grammar;
use crate::util::{self, AsString};

use super::action::Action;
use super::item::Item;

/// The `Table` type represents a simplified version of an automaton.
pub type Table<T> = HashMap<(usize, usize), T>;

/// The `Data` struct contains all automaton data tables.
#[derive(Clone, Debug)]
pub struct Data {
    pub grammar: Grammar,
    pub start_rule: usize,
    pub items: HashMap<usize, Item>,
    pub action_table: Table<Action>,
    pub goto_table: Table<usize>,
    pub left_table: Table<usize>,
    pub backtrack_table: Table<(usize, usize)>,
}

impl Data {
    /// Constructs a new data table.
    pub fn new(
        grammar: &Grammar,
        start_rule: usize,
        items: &[Item],
        action_table: Table<Action>,
        goto_table: Table<usize>,
        left_table: Table<usize>,
        backtrack_table: Table<(usize, usize)>,
    ) -> Data {
        let items = backtrack_table
            .iter()
            .flat_map(|(to, from)| vec![to.1, from.1])
            .map(|id| (id, items[id]))
            .collect();

        Data {
            grammar: grammar.clone(),
            start_rule,
            items,
            action_table,
            goto_table,
            left_table,
            backtrack_table,
        }
    }
}

impl AsString for Data {
    fn string(&self, grammar: &Grammar) -> String {
        let action_table = util::to_sorted_vec(&self.action_table)
            .iter()
            .map(|&(&(state, symbol), action)| {
                format!("{}, {} → {}", state, grammar.symbol(symbol), action)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let goto_table = util::to_sorted_vec(&self.goto_table)
            .iter()
            .map(|&(&(from, symbol), to)| format!("{}, {} → {}", from, grammar.symbol(symbol), to))
            .collect::<Vec<String>>()
            .join("\n");

        let left_table = util::to_sorted_vec(&self.left_table)
            .iter()
            .map(|&(&(state, symbol), item)| {
                format!("{}, {} → {}", state, grammar.symbol(symbol), item)
            })
            .collect::<Vec<String>>()
            .join("\n");

        let backtrack_table = util::to_sorted_vec(&self.backtrack_table)
            .iter()
            .map(|(to, from)| format!("{}, {} → {}, {}", to.0, to.1, from.0, from.1))
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            "ACTION\n{}\n\nGOTO\n{}\n\nLEFT\n{}\n\nBACKTRACK\n{}",
            action_table, goto_table, left_table, backtrack_table
        )
    }
}
