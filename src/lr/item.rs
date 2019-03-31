use std::fmt::{self, Display, Formatter};

use crate::rule::Rule;
use crate::symbol::Symbol;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Item {
    rule: Rule,
    idx: usize,
    follow: Symbol,
}

impl Item {
    pub fn new(rule: Rule, follow: Symbol) -> Item {
        Item {
            rule,
            idx: 0,
            follow,
        }
    }

    pub fn head(&self) -> Option<&Symbol> {
        self.rule.body.get(self.idx)
    }

    pub fn tail(&self) -> Vec<Symbol> {
        let mut result: Vec<Symbol> = self.rule.body[self.idx + 1..].to_vec();
        result.push(self.follow.clone());
        result
    }

    pub fn consume(&mut self) {
        self.idx += 1;
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut body: Vec<String> = self.rule.body.iter().map(Symbol::to_string).collect();

        let pointer = "·".to_owned();
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
            self.follow
        )
    }
}
