use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

#[derive(Debug, PartialEq, Eq)]
pub enum Selection<'a> {
    Found(&'a str),
    Ambiguous(Vec<&'a str>),
    NotFound,
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .map(|c| if c == 'đ' { 'd' } else { c })
        .collect()
}

pub fn select<'a>(items: &'a [&str], query: &str) -> Selection<'a> {
    let query = normalize(query.trim());
    if query.is_empty() {
        return Selection::NotFound;
    }
    let found: Vec<_> = items
        .iter()
        .copied()
        .filter(|item| normalize(item).contains(&query))
        .collect();
    match found.as_slice() {
        [] => Selection::NotFound,
        [one] => Selection::Found(one),
        _ => Selection::Ambiguous(found),
    }
}
