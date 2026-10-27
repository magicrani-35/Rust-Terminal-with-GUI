pub(crate) fn is_valid_identifier(name: &str) -> bool {
    let mut characters = name.chars();

    let valid_start = match characters.next() {
        Some(c) => c.is_ascii_alphabetic() || c == '_',
        None => false,
    };

    valid_start && characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
