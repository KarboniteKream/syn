use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgMatches};

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
pub fn parse_args<'a>() -> ArgMatches<'a> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("input")
                .value_name("INPUT")
                .help("Input file name")
                .required(true),
        )
        .arg(
            Arg::with_name("grammar")
                .long("grammar")
                .short("g")
                .value_name("GRAMMAR")
                .help("Grammar file name")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .value_name("OUTPUT")
                .help("Output file name")
                .takes_value(true),
        )
        .get_matches()
}

/// Returns the index of an element in a Vec.
pub fn get_index<T>(vec: &[T], value: T) -> usize
where
    T: Eq,
{
    vec.iter().position(|item| *item == value).unwrap()
}

/// Converts a collection to a sorted Vec.
pub fn to_sorted_vec<I, T>(collection: I) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    T: Ord,
{
    let mut vec: Vec<T> = collection.into_iter().collect();
    vec.sort_unstable();
    vec
}

/// Calls `AsString.string()` on iterator elements and joins them with `separator`.
pub fn as_string<I, T>(iterator: I, grammar: &Grammar, separator: &str) -> String
where
    I: Iterator<Item = T>,
    T: AsString,
{
    iterator
        .map(|item| item.string(grammar))
        .collect::<Vec<String>>()
        .join(separator)
}
