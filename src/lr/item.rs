use std::fmt::{self, Display, Formatter};

use crate::rule::Rule;
use crate::symbol::Symbol;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Item {
    rule: Rule,
    idx: usize,
    follower: Symbol,
}

impl Item {
    pub fn new(rule: Rule, follower: Symbol) -> Item {
        Item {
            rule,
            idx: 0,
            follower,
        }
    }

    pub fn head(&self) -> Option<&Symbol> {
        self.rule.body.get(self.idx)
    }

    pub fn tail(&self) -> Vec<Symbol> {
        let mut tail: Vec<Symbol> = self.rule.body[self.idx + 1..].to_vec();
        tail.push(self.follower.clone());
        tail
    }

    pub fn pass(&mut self) {
        self.idx += 1;
    }

    pub fn is_nonterminal(&self) -> bool {
        match self.head() {
            Some(head) => head.is_nonterminal(),
            None => false,
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut body: Vec<String> = self.rule.body.iter().map(Symbol::to_string).collect();
        let pointer = String::from("·");

        if let Some(symbol) = body.get_mut(self.idx) {
            *symbol = pointer + symbol;
        } else {
            body.push(pointer);
        }

        write!(
            f,
            "{} → {}, {}",
            self.rule.head,
            body.join(" "),
            self.follower
        )
    }
}
