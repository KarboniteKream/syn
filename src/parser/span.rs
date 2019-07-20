use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Span {
    pub fn new(position: (usize, usize)) -> Span {
        Span {
            start: position,
            end: position,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Span { start, end } = self;

        if start.0 == end.0 {
            if start.1 == end.1 {
                return write!(f, "{}:{}", start.0, start.1);
            }

            return write!(f, "{}:{}-{}", start.0, start.1, end.1);
        }

        write!(f, "{}:{}-{}:{}", start.0, start.1, end.0, end.1)
    }
}
