use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::rule::Rule;
use crate::symbol::Symbol;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: usize,
    pub rule: Rule,
    head: usize,
    lookahead: Symbol,
    pub unique: bool,
}

impl Item {
    pub fn new(id: usize, rule: Rule, lookahead: Symbol, unique: bool) -> Item {
        Item {
            id,
            rule,
            head: 0,
            lookahead,
            unique,
        }
    }

    pub fn head(&self) -> Option<&Symbol> {
        self.rule.body.get(self.head)
    }

    pub fn tail(&self) -> Vec<Symbol> {
        let mut tail: Vec<Symbol> = self.rule.body[self.head + 1..].to_vec();
        tail.push(self.lookahead.clone());
        tail
    }

    pub fn pass(&mut self) {
        self.head += 1;
    }

    pub fn is_nonterminal(&self) -> bool {
        match self.head() {
            Some(head) => head.is_nonterminal(),
            None => false,
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.rule == other.rule
            && self.head == other.head
            && self.lookahead == other.lookahead
            && self.unique == other.unique
    }
}

impl Eq for Item {}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.head.hash(state);
        self.lookahead.hash(state);
        self.unique.hash(state);
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let pointer = String::from("·");

        let mut body = if self.rule.body == vec![Symbol::Null] {
            Vec::new()
        } else {
            self.rule.body.iter().map(Symbol::to_string).collect()
        };

        if let Some(symbol) = body.get_mut(self.head) {
            *symbol = pointer + symbol;
        } else {
            body.push(pointer);
        }

        write!(
            f,
            "{} → {}, {}",
            self.rule.head,
            body.join(" "),
            self.lookahead
        )
    }
}
