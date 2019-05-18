use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::rule::Rule;
use crate::symbol::Symbol;

#[derive(Clone, Debug, Ord, PartialOrd)]
pub struct Item {
    pub id: usize,
    pub rule: Rule,
    pub idx: usize,
    pub lookahead: Symbol,
    pub unique: bool,
}

impl Item {
    pub fn new(id: usize, rule: Rule, lookahead: Symbol, unique: bool) -> Item {
        Item {
            id,
            rule,
            idx: 0,
            lookahead,
            unique,
        }
    }

    pub fn head(&self) -> Option<&Symbol> {
        self.rule.body.get(self.idx)
    }

    pub fn tail(&self) -> Vec<Symbol> {
        let mut tail = if self.head().is_some() {
            self.rule.body[self.idx + 1..].to_vec()
        } else {
            Vec::new()
        };

        tail.push(self.lookahead.clone());
        tail
    }

    pub fn pass(&mut self) {
        if self.idx < self.rule.body.len() {
            self.idx += 1;
        }
    }

    pub fn follow(&self) -> Vec<Symbol> {
        let mut follow = self.rule.body[self.idx..].to_vec();
        follow.push(self.lookahead.clone());
        follow
    }

    pub fn at_nonterminal(&self) -> bool {
        match self.head() {
            Some(head) => head.is_nonterminal(),
            None => false,
        }
    }

    pub fn can_reduce(&self) -> bool {
        if self.rule.id == 0 {
            return false;
        }

        match self.head() {
            Some(head) => *head == Symbol::Null,
            None => true,
        }
    }

    pub fn can_accept(&self) -> bool {
        if self.rule.id != 0 || self.idx == 0 {
            return false;
        }

        match self.head() {
            Some(head) => *head == Symbol::Delimiter,
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

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let dot = String::from("·");

        let mut body = if self.rule.body == vec![Symbol::Null] {
            Vec::new()
        } else {
            self.rule.body.iter().map(Symbol::to_string).collect()
        };

        if let Some(symbol) = body.get_mut(self.idx) {
            *symbol = dot + symbol;
        } else {
            body.push(dot);
        }

        let unique = if self.unique { "○" } else { "×" };

        write!(
            f,
            "({}) {} → {}, {} {}",
            self.id,
            self.rule.head,
            body.join(" "),
            self.lookahead,
            unique
        )
    }
}
