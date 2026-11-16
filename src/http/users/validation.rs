pub fn username(username: String) -> bool {
    if !(3..=20).contains(&username.len()) {
        return false;
    }

    if !username.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '.'
    }) {
        return false;
    }

    true
}