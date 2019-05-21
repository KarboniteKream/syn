use std::hash::{Hash, Hasher};

use crate::grammar::Grammar;
use crate::rule::Rule;
use crate::symbol::Symbol;
use crate::util::AsString;

#[derive(Clone, Debug, Ord, PartialOrd)]
pub struct Item {
    pub id: usize,
    pub rule: usize,
    pub idx: usize,
    pub head: Option<Symbol>,
    pub lookahead: Symbol,
    pub unique: bool,
}

impl Item {
    pub fn new(id: usize, rule: &Rule, lookahead: Symbol, unique: bool) -> Item {
        Item {
            id,
            rule: rule.id,
            idx: 0,
            head: Some(rule.body[0].clone()),
            lookahead,
            unique,
        }
    }

    pub fn pass(&mut self, rule: &Rule) {
        if self.idx < rule.body.len() {
            self.idx += 1;

            self.head = match rule.body.get(self.idx) {
                Some(head) => Some(head.clone()),
                None => None,
            }
        }
    }

    pub fn tail(&self, rule: &Rule) -> Vec<Symbol> {
        let mut tail = if self.head.is_some() {
            rule.body[self.idx + 1..].to_vec()
        } else {
            Vec::new()
        };

        tail.push(self.lookahead.clone());
        tail
    }

    pub fn follow(&self, rule: &Rule) -> Vec<Symbol> {
        let mut follow = rule.body[self.idx..].to_vec();
        follow.push(self.lookahead.clone());
        follow
    }

    pub fn at_nonterminal(&self) -> bool {
        match &self.head {
            Some(head) => head.is_nonterminal(),
            None => false,
        }
    }

    pub fn can_reduce(&self) -> bool {
        if self.rule == 0 {
            return false;
        }

        match &self.head {
            Some(head) => *head == Symbol::Null,
            None => true,
        }
    }

    pub fn can_accept(&self) -> bool {
        if self.rule != 0 || self.idx == 0 {
            return false;
        }

        match &self.head {
            Some(head) => *head == Symbol::End,
            None => false,
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.rule == other.rule
            && self.idx == other.idx
            && self.lookahead == other.lookahead
            && self.unique == other.unique
    }
}

impl Eq for Item {}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.idx.hash(state);
        self.lookahead.hash(state);
        self.unique.hash(state);
    }
}

impl AsString for Item {
    fn as_string(&self, grammar: &Grammar) -> String {
        let rule = grammar.rule(self.rule);
        let dot = String::from("·");

        let mut body = if rule.body == vec![Symbol::Null] {
            Vec::new()
        } else {
            rule.body.iter().map(Symbol::to_string).collect()
        };

        if let Some(symbol) = body.get_mut(self.idx) {
            *symbol = dot + symbol;
        } else {
            body.push(dot);
        }

        let unique = if self.unique { "○" } else { "×" };

        format!(
            "({}) {} → {}, {} {}",
            self.id,
            rule.head,
            body.join(" "),
            self.lookahead,
            unique
        )
    }
}
