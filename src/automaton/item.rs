use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use crate::grammar::{Grammar, Rule, Symbol};
use crate::util::AsString;

/// The `Item` struct describes an item of a state in an automaton.
///
/// An item is unique, if all its parents are unique, and represent the same rule.
/// If an item's parent is the item itself, it's not unique.
///
/// To ensure the relation between `Eq` and `Ord`, the fields
/// in the struct _must_ be unique for a specific `id`.
#[derive(Clone, Copy, Debug, Eq)]
pub struct Item {
    pub id: usize,
    pub rule: usize,
    // Index into the `rule` field.
    pub dot: usize,
    // The symbol the `dot` field refers to.
    pub head: Option<usize>,
    // The symbol that follows the item.
    pub lookahead: usize,
    pub unique: bool,
}

impl Item {
    /// Constructs the initial item.
    pub fn initial(id: usize, rule: &Rule, lookahead: usize) -> Item {
        Item {
            id,
            rule: rule.id,
            dot: 0,
            head: Some(rule.body[0]),
            lookahead,
            unique: true,
        }
    }

    /// Constructs a new item.
    pub fn new(id: usize, rule: &Rule, lookahead: usize, unique: bool) -> Item {
        let mut dot = 0;

        // Ignore the starting $ symbol in wrapper rules.
        if rule.body[dot] == Symbol::End.id() {
            dot += 1;
        }

        Item {
            id,
            rule: rule.id,
            dot,
            head: Some(rule.body[dot]),
            lookahead,
            unique,
        }
    }

    /// Advances `dot` by 1 and updates `head`.
    /// The caller must update `id` to a unique value manually.
    pub fn pass(&mut self, rule: &Rule) {
        if self.dot < rule.body.len() {
            self.dot += 1;
            self.head = rule.body.get(self.dot).cloned();
        }
    }

    /// Returns all symbols following `head`, including the lookahead.
    pub fn tail(&self, rule: &Rule) -> Vec<usize> {
        let mut tail = if self.head.is_some() {
            rule.tail(self.dot + 1).to_vec()
        } else {
            Vec::new()
        };

        tail.push(self.lookahead);
        tail
    }

    /// Returns `head` and all the symbols following it, including the lookahead.
    pub fn follow(&self, rule: &Rule) -> Vec<usize> {
        let mut follow = rule.tail(self.dot).to_vec();
        follow.push(self.lookahead);
        follow
    }

    /// Returns `true` if the head of the item is a nonterminal symbol.
    pub fn at_nonterminal(&self, symbols: &[Symbol]) -> bool {
        match self.head {
            Some(id) => symbols[id].is_nonterminal(),
            None => false,
        }
    }

    /// Returns `true` if the item can reduce the rule.
    pub fn can_reduce(&self, initial_rule: usize) -> bool {
        if self.rule == initial_rule {
            return false;
        }

        match self.head {
            Some(id) => id == Symbol::Null.id(),
            None => true,
        }
    }

    /// Returns `true` if the item can accept the parse stack.
    pub fn can_accept(&self, initial_rule: usize) -> bool {
        self.rule == initial_rule && self.head.is_none()
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.rule == other.rule
            && self.dot == other.dot
            && self.lookahead == other.lookahead
            && self.unique == other.unique
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.dot.hash(state);
        self.lookahead.hash(state);
        self.unique.hash(state);
    }
}

impl AsString for Item {
    fn string(&self, grammar: &Grammar) -> String {
        let rule = grammar.rule(self.rule);
        let dot = String::from("·");

        let mut body = if rule.body == vec![Symbol::Null.id()] {
            Vec::new()
        } else {
            rule.body
                .iter()
                .map(|&id| grammar.symbol(id).to_string())
                .collect()
        };

        if let Some(symbol) = body.get_mut(self.dot) {
            *symbol = dot + symbol;
        } else {
            body.push(dot);
        }

        let unique = if self.unique { "○" } else { "×" };

        format!(
            "({}) {} → {}, {} {}",
            self.id,
            grammar.symbol(rule.head),
            body.join(" "),
            grammar.symbol(self.lookahead),
            unique
        )
    }
}
