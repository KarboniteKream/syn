use std::fmt::Display;

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
