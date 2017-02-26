#![cfg(test)]

/// Converts a `str` to a `Vec<char>`.
pub fn str_to_cvec<T: AsRef<str>>(s: T) -> Vec<char> {
    s.as_ref().chars().collect()
}

/// Converts a slice of `&str`s to a `Vec<Vec<char>>`.
pub fn strs_to_cvecs(strs: &[&str]) -> Vec<Vec<char>> {
    strs.into_iter().map(str_to_cvec).collect()
}
