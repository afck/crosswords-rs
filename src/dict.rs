use cw::BLOCK;
use itertools::Itertools;
use rand;
use rand::Rng;
use std::cmp;
use std::collections::HashMap;
use std::slice;
use std::iter;
use word_constraint::WordConstraint;

fn matches(word: &[char], pattern: &[char]) -> bool {
    word.len() <= pattern.len() &&
    word.iter().zip(pattern.iter()).all(|(&cw, &cp)| cw == cp || cp == BLOCK)
}

/// An iterator over all words satisfying a given `WordConstraint`.
pub struct PatternIter<'a> {
    dict: &'a Dict,
    pattern: Vec<char>,
    list: &'a Vec<usize>,
    index: usize,
}

impl<'a> PatternIter<'a> {
    fn get_word(&self) -> Option<&'a Vec<char>> {
        self.list.get(self.index).and_then(|&i| self.dict.words.get(i))
    }
}

impl<'a> Iterator for PatternIter<'a> {
    type Item = &'a Vec<char>;

    fn next(&mut self) -> Option<&'a Vec<char>> {
        loop {
            let word = self.get_word();
            self.index += 1;
            if word.iter().all(|w| matches(w, &self.pattern)) {
                return word;
            }
        }
    }
}

/// A `Dict` stores a list of words - represented as char vectors - and indexes them for
/// efficiently iterating over all words satisfying a given `WordConstraint`.
pub struct Dict {
    words: Vec<Vec<char>>,
    lists: HashMap<WordConstraint, Vec<usize>>,
    max_n: usize,
    empty_list: Vec<usize>,
}

impl Dict {
    /// Create a new `Dict` from the given sequence of words.
    pub fn new<T, U>(all_words: T) -> Dict
        where T: IntoIterator<Item = U>,
              U: Into<Vec<char>>
    {
        let mut dict = Dict {
            words: all_words.into_iter().map(|w| w.into()).unique().collect(),
            lists: HashMap::new(),
            max_n: 3, // TODO: Make this a parameter?
            empty_list: Vec::new(),
        };
        let mut rng = rand::thread_rng(); // TODO: Make this a parameter
        rng.shuffle(&mut dict.words[..]);
        for (i, word) in dict.words.iter().enumerate() {
            for woco in WordConstraint::all(word, dict.max_n) {
                if !dict.lists.get(&woco).is_some() {
                    dict.lists.insert(woco.clone(), vec![i]);
                } else {
                    dict.lists.get_mut(&woco).unwrap().push(i);
                }
            }
        }
        dict
    }

    fn replace_special(string_word: &str) -> String {
        string_word.replace("Ä", "AE")
            .replace("Ö", "OE")
            .replace("Ü", "UE")
            .replace("ß", "SS")
    }

    /// Convert the `String` to a char vector, replacing umlauts with corresponding diphthongs.
    /// Return `None` if the word contains an invalid character.
    pub fn normalize_word<T: AsRef<str>>(str_word: T) -> Option<Vec<char>> {
        let word = Dict::replace_special(str_word.as_ref().to_uppercase().trim());
        if word.chars().all(|c| c.is_alphabetic()) && !word.is_empty() {
            Some(word.chars().collect())
        } else {
            None
        }
    }

    fn get_list<'a>(&'a self, wc: &WordConstraint) -> &'a Vec<usize> {
        self.lists.get(wc).unwrap_or(&self.empty_list)
    }

    /// Return whether the given word is present in this dictionary.
    pub fn contains(&self, word: &[char]) -> bool {
        self.matching_words(word.clone()).next().is_some()
    }

    /// Return an iterator over all words in the dictionary.
    pub fn all_words(&self) -> slice::Iter<Vec<char>> {
        self.words.iter()
    }

    fn get_matching_word_list(&self, pattern: &[char]) -> &Vec<usize> {
        let len = pattern.len();
        let mut list: &Vec<usize> = self.get_list(&WordConstraint::Length(pattern.len()));
        if list.is_empty() {
            return list;
        }
        let mut pos = 0;
        for i in pattern.iter()
            .enumerate()
            .filter(|&(_, ch)| ch == &BLOCK)
            .map(|(i, _)| i)
            .chain(iter::once(len)) {
            if i > pos {
                let subword = &pattern[pos..i];
                let n = cmp::min(self.max_n, subword.len());
                for dp in 1..(subword.len() - n) {
                    let wc = WordConstraint::with_ngram(&subword[dp..(dp + n)], pos + dp, len);
                    let new_list = self.get_list(&wc);
                    if list.len() > new_list.len() {
                        list = new_list;
                        if list.is_empty() {
                            return list;
                        }
                    }
                }
            }
            pos = i + 1;
        }
        list
    }

    /// Return an iterator over all words in the dictionary matching the given pattern.
    pub fn matching_words(&self, pattern: &[char]) -> PatternIter {
        let list = self.get_matching_word_list(pattern);
        PatternIter {
            dict: self,
            pattern: pattern.to_vec(),
            list: list,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use test_util::*;

    #[test]
    fn test() {
        let dict = Dict::new(strs_to_cvecs(&["FOO", "FOOBAR", "FOE", "TOE"]));
        assert_eq!(2, dict.matching_words(&str_to_cvec("#OE")).count());
        assert_eq!(1, dict.matching_words(&str_to_cvec("F#E")).count());
        assert_eq!(0, dict.matching_words(&str_to_cvec("T#O")).count());
        assert_eq!(0, dict.matching_words(&str_to_cvec("F###")).count());
        assert_eq!(0, dict.matching_words(&str_to_cvec("##")).count());
    }

    #[test]
    fn test_normalize_word() {
        let words = vec!["Öha", "Düsenjäger", "H4X0R", "Wow!", "Fuß"]
            .into_iter()
            .filter_map(Dict::normalize_word)
            .collect_vec();
        let expected = strs_to_cvecs(&["OEHA", "DUESENJAEGER", "FUSS"]);
        assert_eq!(expected, words);
    }
}
