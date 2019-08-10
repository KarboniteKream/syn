use std::collections::HashMap;

use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgMatches};

use crate::grammar::Grammar;

/// The `Table` type represents a simplified version of an automaton.
pub type Table<T> = HashMap<(usize, usize), T>;

/// The `AsString` trait is used as an alternative to the `Display` trait,
/// as it requires a `Grammar` argument to correctly format a struct.
pub trait AsString {
    fn string(&self, grammar: &Grammar) -> String;
}

impl<T: AsString> AsString for &T {
    fn string(&self, grammar: &Grammar) -> String {
        AsString::string(&**self, grammar)
    }
}

/// Parses and validates command-line arguments.
pub fn parse_args<'a>() -> ArgMatches<'a> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("input")
                .value_name("FILE")
                .help("Input file name")
                .required(true),
        )
        .arg(
            Arg::with_name("grammar")
                .long("grammar")
                .short("g")
                .value_name("FILE")
                .help("Grammar file name")
                .required(true),
        )
        .arg(
            Arg::with_name("parser")
                .help("Parser name")
                .long("parser")
                .short("p")
                .value_name("NAME")
                .possible_values(&["lllr", "ll", "lr"])
                .default_value("lllr"),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .value_name("FILE")
                .help("Output file name for the LR automaton"),
        )
        .get_matches()
}

/// Returns the index of an element in a vector.
pub fn get_index<T>(vec: &[T], value: T) -> usize
where
    T: Eq,
{
    vec.iter().position(|item| *item == value).unwrap()
}

/// Converts a collection to a sorted vector.
pub fn to_sorted_vec<I, T>(collection: I) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    T: Ord,
{
    let mut vec: Vec<T> = collection.into_iter().collect();
    vec.sort_unstable();
    vec
}

/// Calls `AsString.string()` on vector elements and joins them with `separator`.
pub fn as_string<T>(vec: &[T], grammar: &Grammar, separator: &str) -> String
where
    T: AsString,
{
    vec.iter()
        .map(|item| item.string(grammar))
        .collect::<Vec<String>>()
        .join(separator)
}
