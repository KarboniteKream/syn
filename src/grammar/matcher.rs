use regex::Regex;

/// The `Matcher` enum describes a method to match an input string.
#[derive(Clone, Debug)]
pub enum Matcher {
    /// Match using a regular expression.
    Regex(Regex),

    /// Compare to a string.
    Text(String),
}

impl Matcher {
    /// Matches an input string against the specified expression.
    pub fn match_str(&self, text: &str) -> Match {
        match self {
            Self::Regex(regex) => {
                let captures = match regex.captures(text) {
                    Some(captures) => captures,
                    None => return Match::None,
                };

                captures
                    .get(captures.len() - 1)
                    .map_or(Match::Partial, |group| {
                        // The last capture group should not be empty.
                        if group.as_str().is_empty() {
                            Match::Partial
                        } else {
                            Match::Full
                        }
                    })
            }
            Self::Text(string) => {
                if string == text {
                    return Match::Full;
                }

                if string.starts_with(text) {
                    return Match::Partial;
                }

                Match::None
            }
        }
    }
}

/// The `Match` enum describes a match type.
pub enum Match {
    /// No match.
    None,

    /// Partial match.
    Partial,

    /// Full match.
    Full,
}
