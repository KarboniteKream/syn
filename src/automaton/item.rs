use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use crate::grammar::Grammar;
use crate::rule::Rule;
use crate::symbol::Symbol;
use crate::util::AsString;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: usize,
    pub rule: usize,
    pub dot: usize,
    pub head: Option<usize>,
    pub lookahead: usize,
    pub unique: bool,
}

impl Item {
    pub fn new(id: usize, rule: &Rule, lookahead: usize, unique: bool) -> Item {
        Item {
            id,
            rule: rule.id,
            dot: 0,
            head: Some(rule.body[0]),
            lookahead,
            unique,
        }
    }

    pub fn pass(&mut self, rule: &Rule) {
        if self.dot < rule.body.len() {
            self.dot += 1;

            self.head = match rule.body.get(self.dot) {
                Some(id) => Some(*id),
                None => None,
            };
        }
    }

    pub fn tail(&self, rule: &Rule) -> Vec<usize> {
        let mut tail = if self.head.is_some() {
            rule.body[self.dot + 1..].to_vec()
        } else {
            Vec::new()
        };

        tail.push(self.lookahead);
        tail
    }

    pub fn follow(&self, rule: &Rule) -> Vec<usize> {
        let mut follow = rule.body[self.dot..].to_vec();
        follow.push(self.lookahead);
        follow
    }

    pub fn at_nonterminal(&self, symbols: &[Symbol]) -> bool {
        match self.head {
            Some(id) => symbols[id].is_nonterminal(),
            None => false,
        }
    }

    pub fn can_reduce(&self) -> bool {
        if self.rule == 0 {
            return false;
        }

        match self.head {
            Some(id) => id == Symbol::Null.id(),
            None => true,
        }
    }

    pub fn can_accept(&self) -> bool {
        if self.rule != 0 || self.dot == 0 {
            return false;
        }

        match self.head {
            Some(id) => id == Symbol::End.id(),
            None => false,
        }
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

impl Eq for Item {}

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
                .map(|id| grammar.symbol(*id).to_string())
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
