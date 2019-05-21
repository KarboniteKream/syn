use std::fmt::Display;

use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgMatches};

use crate::grammar::Grammar;

pub trait AsString {
    fn as_string(&self, grammar: &Grammar) -> String;
}

impl<T: AsString> AsString for &T {
    fn as_string(&self, grammar: &Grammar) -> String {
        AsString::as_string(&**self, grammar)
    }
}

pub fn parse_args<'a>() -> ArgMatches<'a> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("filename")
                .value_name("INPUT")
                .help("Grammar file name")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .value_name("FILE")
                .help("Output file name")
                .takes_value(true),
        )
        .get_matches()
}

pub fn to_sorted_vec<I, T>(collection: &I) -> Vec<T>
where
    I: Clone + IntoIterator<Item = T>,
    T: Ord,
{
    let mut vec: Vec<T> = collection.clone().into_iter().collect();
    vec.sort_unstable();
    vec
}

pub fn to_string<I, T>(iterator: I, separator: &str) -> String
where
    I: Iterator<Item = T>,
    T: Display,
{
    iterator
        .map(|item| item.to_string())
        .collect::<Vec<String>>()
        .join(separator)
}

pub fn as_string<I, T>(iterator: I, grammar: &Grammar, separator: &str) -> String
where
    I: Iterator<Item = T>,
    T: AsString,
{
    iterator
        .map(|item| item.as_string(grammar))
        .collect::<Vec<String>>()
        .join(separator)
}
