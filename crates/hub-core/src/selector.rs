use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

/// Result of picking one item by a spoken label.
#[derive(Debug, PartialEq, Eq)]
pub enum Selection<'a> {
    /// Index of the chosen item.
    Found(usize),
    Ambiguous(Vec<&'a str>),
    NotFound,
}

fn normalize(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .map(|c| if c == 'đ' { 'd' } else { c })
        .collect()
}

/// Picks the item whose label matches `query`, ignoring case and Vietnamese
/// diacritics. An exact match wins over labels that only contain the query.
pub fn select<'a>(items: &[&'a str], query: &str) -> Selection<'a> {
    let query = normalize(query);
    if query.is_empty() {
        return Selection::NotFound;
    }
    let labels: Vec<_> = items.iter().map(|item| normalize(item)).collect();
    let exact: Vec<_> = (0..items.len()).filter(|&i| labels[i] == query).collect();
    let found = if exact.is_empty() {
        (0..items.len())
            .filter(|&i| labels[i].contains(&query))
            .collect()
    } else {
        exact
    };
    match found.as_slice() {
        [] => Selection::NotFound,
        [one] => Selection::Found(*one),
        _ => Selection::Ambiguous(found.iter().map(|&i| items[i]).collect()),
    }
}

#[cfg(test)]
#[path = "selector_tests.rs"]
mod tests;
