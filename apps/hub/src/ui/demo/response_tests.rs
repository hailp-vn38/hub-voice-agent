use super::split_words;

#[test]
fn marks_words_of_emphasis_phrase() {
    let w = split_words("Còn 7 phút nữa là xong", &["7 phút"]);
    let flags: Vec<bool> = w.iter().map(|(_, e)| *e).collect();
    assert_eq!(w.len(), 7);
    assert_eq!(flags, [false, true, true, false, false, false, false]);
    assert_eq!(w[1].0, "7");
}

#[test]
fn no_emphasis_when_phrase_absent() {
    let w = split_words("Xin chào", &["7 phút"]);
    assert!(w.iter().all(|(_, e)| !*e));
}
