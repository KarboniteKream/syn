use clap::{Arg, ArgMatches, Command, crate_name, crate_version};

use crate::grammar::Grammar;

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
pub fn parse_args() -> ArgMatches {
    Command::new(crate_name!())
        .version(crate_version!())
        .arg(
            Arg::new("input")
                .value_name("INPUT")
                .help("Input file name")
                .required(true),
        )
        .arg(
            Arg::new("grammar")
                .long("grammar")
                .short('g')
                .value_name("FILE")
                .help("Grammar file name")
                .required(true),
        )
        .arg(
            Arg::new("parser")
                .help("Parser name")
                .long("parser")
                .short('p')
                .value_name("NAME")
                .value_parser(["lllr", "ll", "lr"])
                .default_value("lllr"),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
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
