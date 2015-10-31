#![cfg(test)]

/// Converts a `&str` to a `Vec<char>`.
pub fn str_to_cvec(s: &str) -> Vec<char> {
    s.chars().collect()
}

