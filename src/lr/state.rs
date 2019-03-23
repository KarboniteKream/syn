use std::fmt::{self, Display, Formatter};

use super::item::Item;

#[derive(Debug)]
pub struct State {
    pub items: Vec<Item>,
}

impl State {
    pub fn new(items: Vec<Item>) -> State {
        State { items }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let items = self
            .items
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join(";  ");

        write!(f, "[{}]", items)
    }
}
